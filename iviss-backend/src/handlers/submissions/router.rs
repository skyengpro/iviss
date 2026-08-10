use crate::app_state::AppState;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;

/// Admin-only pending submission review routes.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/admin/submissions",
            get(super::list_pending_submissions),
        )
        .route(
            "/api/v1/admin/submissions/:id",
            get(super::get_pending_submission),
        )
        .route(
            "/api/v1/admin/submissions/:id/audit",
            get(super::get_submission_audit_log),
        )
}

/// Routes for any authenticated user (agent submitting an unregistered vehicle).
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/vehicles/pending", post(super::submit_vehicle_v1))
}
