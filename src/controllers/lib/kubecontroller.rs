#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
#![allow(unused_imports, unused_variables)]
use crate::controllers::{
    lib::{ErrorWrapper, LocoErrorExt, Result, telemetry::Diagnostics},
    metrics::Metrics,
    telemetry,
};
use loco_rs::Error as LocoError;

use chrono::{DateTime, Utc};
use futures::StreamExt;
use kube::{
    CustomResource, Resource,
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        events::{Event, EventType, Recorder, Reporter},
        finalizer::{Event as Finalizer, finalizer},
        watcher::Config,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::{sync::RwLock, time::Duration};
use tracing::{Callsite, Span, Subscriber, Value, field, info, instrument, warn};

pub static DOCUMENT_FINALIZER: &str = "documents.kube.rs";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[cfg_attr(test, derive(Default))]
#[kube(kind = "Document", group = "kube.rs", version = "v1", namespaced)]
#[kube(status = "DocumentStatus", shortname = "doc")]
pub struct DocumentSpec {
    pub title: String,
    pub hide: bool,
    pub content: String,
}


#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct DocumentStatus {
    pub hidden: bool,
}

impl Document {
    #[allow(dead_code)]
    fn was_hidden(&self) -> bool {
        self.status.as_ref().is_some_and(|s| s.hidden)
    }
}

#[derive(Clone)]
pub struct Context {
    pub client: Client,
    pub recorder: Recorder,
    pub diagnostics: Arc<RwLock<Diagnostics>>,
    pub metrics: Arc<Metrics>,
}

#[instrument(skip(ctx, doc), fields(trace_id, document = ?doc.name_any()))]
async fn reconcile(doc: Arc<Document>, ctx: Arc<Context>) -> Result<Action> {
    let trace_id = telemetry::get_trace_id();
    if trace_id != opentelemetry::trace::TraceId::INVALID {
        Span::current().record("trace_id", field::display(&trace_id));
    }

    let _timer = ctx.metrics.reconcile.count_and_measure(&trace_id);
    ctx.diagnostics.write().await.last_event = Utc::now();

    let Some(ns) = doc.namespace() else { return Err(ErrorWrapper::from_custom("Document namespace is missing")) };

    let docs: Api<Document> = Api::namespaced(ctx.client.clone(), &ns);

    info!(document_name = %doc.name_any(), namespace = %ns, "Reconciling Document");

    finalizer(&docs, DOCUMENT_FINALIZER, doc, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
    .await
        .map_err(ErrorWrapper::from_kube)
}

#[allow(dead_code)]
fn error_policy(doc: &Arc<Document>, error: &LocoError, ctx: &Arc<Context>) -> Action {
    warn!("reconcile failed: {:?}", error);
    ctx.metrics.reconcile.set_failure(doc, error); // `error` is now `LocoError`
    Action::requeue(Duration::from_secs(5 * 60))
}

impl Document {

    #[allow(dead_code)]
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action> {
        let client = ctx.client.clone();
        let oref = self.object_ref(&());
        let ns = self.namespace().unwrap();
        let name = self.name_any();
        let docs: Api<Self> = Api::namespaced(client, &ns);

        let should_hide = self.spec.hide;
        if !self.was_hidden() && should_hide {
            ctx.recorder
                .publish(
                    &Event {
                        type_: EventType::Normal,
                        reason: "HideRequested".into(),
                        note: Some(format!("Hiding `{name}`")),
                        action: "Hiding".into(),
                        secondary: None,
                    },
                    &oref,
                )
                .await
                .map_err(ErrorWrapper::from_kube)?;
        }
        if name == "illegal" {
            return Err(ErrorWrapper::from_custom("IllegalDocument"));
        }

        let new_status = Patch::Apply(json!({
            "apiVersion": "kube.rs/v1",
            "kind": "Document",
            "status": DocumentStatus {
                hidden: should_hide,
            }
        }));
        let ps = PatchParams::apply("cntrlr").force();
        let _o = docs
            .patch_status(&name, &ps, &new_status)
            .await
            .map_err(ErrorWrapper::from_kube);

        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    #[allow(dead_code)]
    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action> {
        let oref = self.object_ref(&());
        ctx.recorder
            .publish(
                &Event {
                    type_: EventType::Normal,
                    reason: "DeleteRequested".into(),
                    note: Some(format!("Delete `{}`", self.name_any())),
                    action: "Deleting".into(),
                    secondary: None,
                },
                &oref,
            )
            .await
            .map_err(ErrorWrapper::from_kube)?;
        Ok(Action::await_change())
    }
}

