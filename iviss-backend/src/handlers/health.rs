use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use std::sync::Arc;
use tracing::instrument;

use crate::app_state::AppState;

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "health",
    operation_id = "metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = String)
    )
)]
pub async fn metrics_export(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        state.telemetry.metrics_output(),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    operation_id = "health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    )
)]
#[instrument(name = "health.check")]
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
