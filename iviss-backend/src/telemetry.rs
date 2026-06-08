use anyhow::Result;
use metrics_exporter_prometheus::PrometheusBuilder;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct TelemetryHandle {
    pub metrics_recorder: Arc<metrics_exporter_prometheus::PrometheusHandle>,
    tracer_provider: Option<TracerProvider>,
}

impl TelemetryHandle {
    pub fn metrics_output(&self) -> String {
        self.metrics_recorder.render()
    }

    pub fn shutdown(&self) {
        if let Some(tp) = &self.tracer_provider {
            if let Err(e) = tp.shutdown() {
                tracing::error!("error shutting down tracer provider: {e}");
            }
        }
    }
}

pub fn init_telemetry(log_level: &crate::config::LogLevel) -> Result<TelemetryHandle> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level.as_tracing_level().to_string()));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .compact();

    let (tracer_provider, otel_layer) = match init_tracer_provider() {
        Ok(provider) => {
            let tracer = provider.tracer("iviss-backend");
            let layer = tracing_opentelemetry::layer().with_tracer(tracer);
            (Some(provider), Some(layer))
        }
        Err(e) => {
            eprintln!("OTel tracer init failed (traces disabled): {e}");
            (None, None)
        }
    };

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    match otel_layer {
        Some(layer) => registry.with(layer).init(),
        None => registry.init(),
    }

    let metrics_handle = init_metrics()?;

    let handle = TelemetryHandle {
        metrics_recorder: Arc::new(metrics_handle),
        tracer_provider,
    };

    tracing::info!("Telemetry initialized (metrics + tracing)");

    Ok(handle)
}

fn init_tracer_provider() -> Result<TracerProvider> {
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4318".to_string());

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "iviss-backend".to_string());

    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", service_name),
        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(&otlp_endpoint)
        .build()?;

    let tracer_provider = TracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    Ok(tracer_provider)
}

fn init_metrics() -> Result<metrics_exporter_prometheus::PrometheusHandle> {
    let handle = PrometheusBuilder::new()
        .set_quantiles(&[0.5, 0.9, 0.95, 0.99])?
        .install_recorder()?;

    metrics::describe_counter!("iviss_http_requests_total", "Total number of HTTP requests");
    metrics::describe_histogram!(
        "iviss_http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    metrics::describe_gauge!("iviss_active_sessions", "Number of active sessions");
    metrics::describe_counter!("iviss_scans_total", "Total number of plate scans");
    metrics::describe_counter!("iviss_scan_errors_total", "Total number of scan errors");
    metrics::describe_counter!(
        "iviss_auth_attempts_total",
        "Total number of authentication attempts"
    );
    metrics::describe_counter!(
        "iviss_auth_failures_total",
        "Total number of authentication failures"
    );

    Ok(handle)
}

pub async fn metrics_handler(handle: Arc<TelemetryHandle>) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        handle.metrics_output(),
    )
}
