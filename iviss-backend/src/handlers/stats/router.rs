use crate::app_state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

/// Admin-only dashboard stats.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/admin/stats", get(super::get_dashboard_stats))
}

/// Org-admin scoped dashboard stats.
pub fn org_admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/org-admin/stats",
            get(super::get_org_dashboard_stats),
        )
        .route(
            "/api/v1/org-admin/activity-feed",
            get(super::get_org_activity_feed),
        )
        .route(
            "/api/v1/org-admin/recent-alerts",
            get(super::get_org_recent_alerts),
        )
        .route(
            "/api/v1/org-admin/top-agents",
            get(super::get_org_top_agents),
        )
        .route(
            "/api/v1/org-admin/activity",
            get(super::get_org_control_activity),
        )
}

/// Routes for any authenticated user (agent, manager, admin, org_admin).
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/stats/activity", get(super::get_control_activity))
        .route("/api/v1/stats/top-agents", get(super::get_top_agents))
        .route("/api/v1/stats/activity-feed", get(super::get_activity_feed))
        .route("/api/v1/stats/recent-alerts", get(super::get_recent_alerts))
}
