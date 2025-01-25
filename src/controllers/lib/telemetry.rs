#![allow(unused_imports)]

use chrono::{DateTime, Utc};
// some used only for telemetry feature
use opentelemetry::trace::{TraceId, TracerProvider};
use opentelemetry_sdk::{Resource, runtime, trace as sdktrace, trace::Config};
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

//
// #[derive(Default)]
// pub struct Diagnostics {
//     pub last_event: DateTime<Utc>,
// }

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

    opentelemetry::global::set_tracer_provider(provider.clone());
    provider.tracer("tracing-otel-subscriber")
}

/// Initializes the telemetry and logging system.
///
/// # Panics
///
/// This function will panic if it fails to create an `EnvFilter` from the default environment or fallback to 'info'.
#[allow(clippy::or_fun_call)]
#[allow(clippy::unused_async)]
pub async fn init() {
    // Setup tracing layers
    #[cfg(feature = "telemetry")]
    let otel = tracing_opentelemetry::OpenTelemetryLayer::new(init_tracer());

    let logger = tracing_subscriber::fmt::layer().compact();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("Failed to create EnvFilter from default environment or fallback to 'info'");

    let reg = Registry::default();
    #[cfg(feature = "telemetry")]
    reg.with(env_filter).with(logger).with(otel).init();
    #[cfg(not(feature = "telemetry"))]
    reg.with(env_filter).with(logger).init();
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
