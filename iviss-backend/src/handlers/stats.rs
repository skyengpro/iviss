use crate::app_state::AppState;
use crate::dto::stats::{
    ActivityFeedResponse, ControlActivityResponse, DashboardRange, RecentAlertsResponse,
    TopAgentsResponse,
};
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::errors::AppError;

// ── GET /stats ────────────────────────────────────────────────────────────────

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
    let stats = crate::queries::stats_queries::get_dashboard_stats_query(&state.db).await?;
    Ok((StatusCode::OK, Json(stats)))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ActivityQuery {
    pub range: Option<DashboardRange>,
}

// ── GET /stats/activity ───────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/stats/activity",
    tag = "stats",
    operation_id = "getControlActivity",
    params(ActivityQuery),
    responses(
        (status = 200, description = "Control activity series retrieved successfully", body = ControlActivityResponse),
        (status = 401, description = "Unauthorized",                                body = AppErrorResponse),
        (status = 500, description = "Internal server error",                       body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_control_activity(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ActivityQuery>,
) -> Result<impl IntoResponse, AppError> {
    let range = query.range.unwrap_or(DashboardRange::H24);
    let series =
        crate::queries::stats_queries::get_control_activity_series_query(&state.db, range).await?;
    Ok((
        StatusCode::OK,
        Json(ControlActivityResponse { range, series }),
    ))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TopAgentsQuery {
    pub range: Option<DashboardRange>,
    pub limit: Option<i64>,
}

// ── GET /stats/top-agents ─────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/stats/top-agents",
    tag = "stats",
    operation_id = "getTopAgents",
    params(TopAgentsQuery),
    responses(
        (status = 200, description = "Top agents retrieved successfully", body = TopAgentsResponse),
        (status = 401, description = "Unauthorized",                     body = AppErrorResponse),
        (status = 500, description = "Internal server error",            body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_top_agents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopAgentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let range = query.range.unwrap_or(DashboardRange::H24);
    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let agents =
        crate::queries::stats_queries::get_top_agents_query(&state.db, range, limit).await?;
    Ok((StatusCode::OK, Json(TopAgentsResponse { range, agents })))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFeedQuery {
    pub limit: Option<i64>,
}

// ── GET /stats/activity-feed ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/stats/activity-feed",
    tag = "stats",
    operation_id = "getActivityFeed",
    params(ActivityFeedQuery),
    responses(
        (status = 200, description = "Activity feed retrieved successfully", body = ActivityFeedResponse),
        (status = 401, description = "Unauthorized",                        body = AppErrorResponse),
        (status = 500, description = "Internal server error",               body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_activity_feed(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ActivityFeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(8).clamp(1, 20);
    let items = crate::queries::stats_queries::get_activity_feed_query(&state.db, limit).await?;
    Ok((StatusCode::OK, Json(ActivityFeedResponse { items })))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct RecentAlertsQuery {
    pub limit: Option<i64>,
}

// ── GET /stats/recent-alerts ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/stats/recent-alerts",
    tag = "stats",
    operation_id = "getRecentAlerts",
    params(RecentAlertsQuery),
    responses(
        (status = 200, description = "Recent alerts retrieved successfully", body = RecentAlertsResponse),
        (status = 401, description = "Unauthorized",                             body = AppErrorResponse),
        (status = 500, description = "Internal server error",                    body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_recent_alerts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RecentAlertsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let items = crate::queries::stats_queries::get_recent_alerts_query(&state.db, limit).await?;
    Ok((StatusCode::OK, Json(RecentAlertsResponse { items })))
}
