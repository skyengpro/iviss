use axum::{http::StatusCode, response::IntoResponse};

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    operation_id = "health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    )
)]
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
