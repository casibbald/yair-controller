#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use crate::core::{LocoErrorExt, kubecontroller::Document};
use axum::debug_handler;
use kube::ResourceExt;
use loco_rs::{Error as LocoError, prelude::*};
use opentelemetry::trace::TraceId;
use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{counter::Counter, exemplar::HistogramWithExemplars, family::Family},
    registry::{Registry, Unit},
};
use std::sync::Arc;
use tokio::time::Instant;

#[debug_handler]
pub async fn index(State(_ctx): State<AppContext>) -> Result<Response> {
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/metrics/").add("/", get(index))
}


#[derive(Clone)]
pub struct Metrics {
    pub reconcile: ReconcileMetrics,
    pub registry: Arc<Registry>,
}

impl Default for Metrics {
    fn default() -> Self {
        let mut registry = Registry::with_prefix("doc_ctrl_reconcile");
        let reconcile = ReconcileMetrics::default().register(&mut registry);
        Self {
            registry: Arc::new(registry),
            reconcile,
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug, Default)]
pub struct TraceLabel {
    pub trace_id: String,
}
impl TryFrom<&TraceId> for TraceLabel {
    type Error = anyhow::Error;

    #[allow(clippy::redundant_else)]
    fn try_from(id: &TraceId) -> Result<Self, Self::Error> {
        if std::matches!(id, &TraceId::INVALID) {
            anyhow::bail!("invalid trace id")
        } else {
            let trace_id = id.to_string();
            Ok(Self { trace_id })
        }
    }
}

#[derive(Clone)]
pub struct ReconcileMetrics {
    pub runs: Counter,
    pub failures: Family<ErrorLabels, Counter>,
    pub duration: HistogramWithExemplars<TraceLabel>,
}

impl Default for ReconcileMetrics {
    fn default() -> Self {
        Self {
            runs: Counter::default(),
            failures: Family::<ErrorLabels, Counter>::default(),
            duration: HistogramWithExemplars::new([0.01, 0.1, 0.25, 0.5, 1., 5., 15., 60.].into_iter()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ErrorLabels {
    pub instance: String,
    pub error: String,
}

impl ReconcileMetrics {
    #[must_use]
    pub fn register(self, r: &mut Registry) -> Self {
        r.register_with_unit(
            "duration",
            "reconcile duration",
            Unit::Seconds,
            self.duration.clone(),
        );
        r.register("failures", "reconciliation errors", self.failures.clone());
        r.register("runs", "reconciliations", self.runs.clone());
        self
    }

    pub fn set_failure(&self, doc: &Document, e: &LocoError) {
        self.failures
            .get_or_create(&ErrorLabels {
                instance: doc.name_any(),
                error: e.metric_label(),
            })
            .inc();
    }

    #[must_use]
    pub fn count_and_measure(&self, trace_id: &TraceId) -> ReconcileMeasurer {
        self.runs.inc();
        ReconcileMeasurer {
            start: Instant::now(),
            labels: trace_id.try_into().ok(),
            metric: self.duration.clone(),
        }
    }
}


pub struct ReconcileMeasurer {
    start: Instant,
    labels: Option<TraceLabel>,
    metric: HistogramWithExemplars<TraceLabel>,
}

impl Drop for ReconcileMeasurer {
    fn drop(&mut self) {
        #[allow(clippy::cast_precision_loss)]
        let duration = self.start.elapsed().as_millis() as f64 / 1000.0;
        let labels = self.labels.take();
        self.metric.observe(duration, labels);
    }
}
