use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::errors::AppError;

// ── GET /admin/stats ─────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    tag = "stats",
    operation_id = "getDashboardStats",
    responses(
        (status = 200, description = "Dashboard statistics retrieved successfully", body = DashboardStats),
        (status = 401, description = "Unauthorized",                                body = AppErrorResponse),
        (status = 500, description = "Internal server error",                       body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_stats(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let stats = crate::queries::stats::get_dashboard_stats_query(&state.db).await?;
    Ok((StatusCode::OK, Json(stats)))
}
