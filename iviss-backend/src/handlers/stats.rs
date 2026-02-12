use axum::{http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::{
    dto::stats::DashboardStats,
    errors::{AppError, AppErrorResponse},
};

// ── GET /stats ────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/stats",
    tag = "stats",
    responses(
        (status = 200, description = "Dashboard statistics retrieved successfully", body = DashboardStats),
        (status = 401, description = "Unauthorized",                                body = AppErrorResponse),
        (status = 500, description = "Internal server error",                       body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_stats() -> Result<impl IntoResponse, AppError> {
    // TODO: Query actual stats from DB via stats_service
    // For now return stub data

    let stats = DashboardStats {
        today_controls: 42,
        active_alerts: 3,
        total_vehicles: 15_247,
        online_agents: 8,
    };

    Ok((StatusCode::OK, Json(stats)))
}
