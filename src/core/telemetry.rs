#![allow(unused_imports)]

use chrono::{DateTime, Utc};
use opentelemetry::trace::{TraceId, TracerProvider};
use opentelemetry_sdk::{
    Resource, runtime, trace as sdktrace,
    trace::{Config, Tracer},
};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

#[must_use]
pub fn get_trace_id() -> TraceId {
    use opentelemetry::trace::TraceContextExt as _;
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    tracing::Span::current()
        .context()
        .span()
        .span_context()
        .trace_id()
}

#[cfg(feature = "telemetry")]
fn resource() -> Resource {
    use opentelemetry::KeyValue;
    Resource::new([
        KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ])
}

#[cfg(feature = "telemetry")]
fn init_tracer() -> sdktrace::Tracer {
    use opentelemetry_otlp::{SpanExporter, WithExportConfig};
    let endpoint = std::env::var("OPENTELEMETRY_ENDPOINT_URL").expect("Needs an otel collector");
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .unwrap();

    let provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(resource())
        .build();

    // opentelemetry::global::set_tracer_provider(provider.clone());
    provider.tracer("tracing-otel-subscriber")
}


static TELEMETRY_INITIALIZED: AtomicBool = AtomicBool::new(false);
#[allow(clippy::or_fun_call)]
#[allow(clippy::unused_async)]
pub async fn init() {
    // Check if telemetry has already been initialized
    if TELEMETRY_INITIALIZED.load(Ordering::SeqCst) {
        tracing::info!("Telemetry already initialized, skipping...");
        return;
    }

    // Setup tracing layers
    #[cfg(feature = "telemetry")]
    let otel = tracing_opentelemetry::OpenTelemetryLayer::new(init_tracer());

    let logger = tracing_subscriber::fmt::layer().compact();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("Failed to create EnvFilter from default environment or fallback to 'info'");

    let reg = Registry::default();

    let subscriber = reg.with(env_filter).with(logger);
    #[cfg(feature = "telemetry")]
    let subscriber = subscriber.with(otel);

    // Set the global default subscriber
    if let Err(_) = tracing::subscriber::set_global_default(subscriber) {
        tracing::warn!("Global default subscriber already set!");
    } else {
        tracing::info!("Initialized telemetry");
    }

    // Mark telemetry as initialized
    TELEMETRY_INITIALIZED.store(true, Ordering::SeqCst);
}


#[cfg(test)]
mod test {
    // This test only works when telemetry is initialized fully
    // and requires OPENTELEMETRY_ENDPOINT_URL pointing to a valid server
    #[cfg(feature = "telemetry")]
    #[tokio::test]
    #[ignore = "requires a trace exporter"]
    async fn get_trace_id_returns_valid_traces() {
        use super::*;
        super::init().await;
        #[tracing::instrument(name = "test_span")] // need to be in an instrumented fn
        fn test_trace_id() -> TraceId {
            get_trace_id()
        }
        assert_ne!(test_trace_id(), TraceId::INVALID, "valid trace");
    }
}
