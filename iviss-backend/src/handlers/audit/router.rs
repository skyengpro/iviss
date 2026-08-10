use crate::app_state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

/// Admin-only audit log routes.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/admin/audit", get(super::list_audit_logs))
        .route("/api/v1/admin/audit/export", get(super::export_audit_logs))
}
