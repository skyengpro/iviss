use axum::{http::StatusCode, response::IntoResponse};
use tracing::instrument;

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
