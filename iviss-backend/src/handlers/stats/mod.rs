use crate::app_state::AppState;
use crate::dto::stats::{
    ActivityFeedQuery, ActivityFeedResponse, ActivityQuery, ControlActivityResponse,
    DashboardRange, RecentAlertsQuery, RecentAlertsResponse, TopAgentsQuery, TopAgentsResponse,
};
use crate::middleware::rbac::AuthenticatedAdmin;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
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
    let stats = crate::queries::stats::get_dashboard_stats_query(&state.db).await?;
    Ok((StatusCode::OK, Json(stats)))
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
    let series = crate::queries::stats::get_control_activity_series_query(&state.db, range).await?;
    Ok((
        StatusCode::OK,
        Json(ControlActivityResponse { range, series }),
    ))
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
    let agents = crate::queries::stats::get_top_agents_query(&state.db, range, limit).await?;
    Ok((StatusCode::OK, Json(TopAgentsResponse { range, agents })))
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
    let items = crate::queries::stats::get_activity_feed_query(&state.db, limit).await?;
    Ok((StatusCode::OK, Json(ActivityFeedResponse { items })))
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
    let items = crate::queries::stats::get_recent_alerts_query(&state.db, limit).await?;
    Ok((StatusCode::OK, Json(RecentAlertsResponse { items })))
}

// ══════════════════════════════════════════════════════════════════════════════
// Org-scoped handlers — for org_admin users
// ══════════════════════════════════════════════════════════════════════════════

// ── GET /org-admin/stats ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/org-admin/stats",
    tag = "org-admin",
    operation_id = "getOrgDashboardStats",
    responses(
        (status = 200, description = "Org dashboard statistics retrieved successfully", body = OrgDashboardStats),
        (status = 401, description = "Unauthorized",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",     body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_org_dashboard_stats(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<AuthenticatedAdmin>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = admin
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;
    let stats = crate::queries::stats::get_org_dashboard_stats_query(&state.db, org_id).await?;
    Ok((StatusCode::OK, Json(stats)))
}

// ── GET /org-admin/activity-feed ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/org-admin/activity-feed",
    tag = "org-admin",
    operation_id = "getOrgActivityFeed",
    params(ActivityFeedQuery),
    responses(
        (status = 200, description = "Org activity feed retrieved successfully", body = ActivityFeedResponse),
        (status = 401, description = "Unauthorized",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",     body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_org_activity_feed(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<AuthenticatedAdmin>,
    Query(query): Query<ActivityFeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = admin
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;
    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let items =
        crate::queries::stats::get_org_activity_feed_query(&state.db, org_id, limit).await?;
    Ok((StatusCode::OK, Json(ActivityFeedResponse { items })))
}

// ── GET /org-admin/recent-alerts ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/org-admin/recent-alerts",
    tag = "org-admin",
    operation_id = "getOrgRecentAlerts",
    params(RecentAlertsQuery),
    responses(
        (status = 200, description = "Org recent alerts retrieved successfully", body = RecentAlertsResponse),
        (status = 401, description = "Unauthorized",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",     body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_org_recent_alerts(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<AuthenticatedAdmin>,
    Query(query): Query<RecentAlertsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = admin
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;
    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let items =
        crate::queries::stats::get_org_recent_alerts_query(&state.db, org_id, limit).await?;
    Ok((StatusCode::OK, Json(RecentAlertsResponse { items })))
}

// ── GET /org-admin/top-agents ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/org-admin/top-agents",
    tag = "org-admin",
    operation_id = "getOrgTopAgents",
    params(TopAgentsQuery),
    responses(
        (status = 200, description = "Org top agents retrieved successfully", body = TopAgentsResponse),
        (status = 401, description = "Unauthorized",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",     body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_org_top_agents(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<AuthenticatedAdmin>,
    Query(query): Query<TopAgentsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = admin
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;
    let range = query.range.unwrap_or(DashboardRange::H24);
    let limit = query.limit.unwrap_or(5).clamp(1, 20);
    let agents =
        crate::queries::stats::get_org_top_agents_query(&state.db, org_id, range, limit).await?;
    Ok((StatusCode::OK, Json(TopAgentsResponse { range, agents })))
}

// ── GET /org-admin/activity ───────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/org-admin/activity",
    tag = "org-admin",
    operation_id = "getOrgControlActivity",
    params(ActivityQuery),
    responses(
        (status = 200, description = "Org control activity series retrieved successfully", body = ControlActivityResponse),
        (status = 401, description = "Unauthorized",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",     body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_org_control_activity(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<AuthenticatedAdmin>,
    Query(query): Query<ActivityQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = admin
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;
    let range = query.range.unwrap_or(DashboardRange::H24);
    let series =
        crate::queries::stats::get_org_control_activity_series_query(&state.db, org_id, range)
            .await?;
    Ok((
        StatusCode::OK,
        Json(ControlActivityResponse { range, series }),
    ))
}
