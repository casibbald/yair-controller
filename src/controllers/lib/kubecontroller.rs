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
use tracing::*;

pub static DOCUMENT_FINALIZER: &str = "documents.kube.rs";

/// Generate the Kubernetes wrapper struct `Document` from our Spec and Status struct
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[cfg_attr(test, derive(Default))]
#[kube(kind = "Document", group = "kube.rs", version = "v1", namespaced)]
#[kube(status = "DocumentStatus", shortname = "doc")]
pub struct DocumentSpec {
    pub title: String,
    pub hide: bool,
    pub content: String,
}

/// The status object of `Document`
#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct DocumentStatus {
    pub hidden: bool,
}

impl Document {
    fn was_hidden(&self) -> bool {
        self.status.as_ref().map(|s| s.hidden).unwrap_or(false)
    }
}

// Context for our reconciler
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
    /// Event recorder
    pub recorder: Recorder,
    /// Diagnostics read by the web server
    pub diagnostics: Arc<RwLock<Diagnostics>>,
    /// Prometheus metrics
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

    let ns = match doc.namespace() {
        Some(ns) => ns,
        None => return Err(ErrorWrapper::from_custom("Document namespace is missing")),
    };

    let docs: Api<Document> = Api::namespaced(ctx.client.clone(), &ns);

    info!(document_name = %doc.name_any(), namespace = %ns, "Reconciling Document");

    finalizer(&docs, DOCUMENT_FINALIZER, doc, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| ErrorWrapper::from_kube(e))
}

fn error_policy(doc: Arc<Document>, error: &LocoError, ctx: Arc<Context>) -> Action {
    warn!("reconcile failed: {:?}", error);
    ctx.metrics.reconcile.set_failure(&doc, error); // `error` is now `LocoError`
    Action::requeue(Duration::from_secs(5 * 60))
}

impl Document {
    // Reconcile (for non-finalizer related changes)
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action> {
        let client = ctx.client.clone();
        let oref = self.object_ref(&());
        let ns = self.namespace().unwrap();
        let name = self.name_any();
        let docs: Api<Document> = Api::namespaced(client, &ns);

        let should_hide = self.spec.hide;
        if !self.was_hidden() && should_hide {
            // send an event once per hide
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
                .map_err(|e| ErrorWrapper::from_kube(e))?;
        }
        if name == "illegal" {
            return Err(ErrorWrapper::from_custom("IllegalDocument")); // error names show up in metrics
        }
        // always overwrite status object with what we saw
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
            .map_err(|e| ErrorWrapper::from_kube(e))?;

        // If no events were received, check back every 5 minutes
        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    // Finalizer cleanup (the object was deleted, ensure nothing is orphaned)
    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action> {
        let oref = self.object_ref(&());
        // Document doesn't have any real cleanup, so we just publish an event
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

// Diagnostics and State remain unchanged.
