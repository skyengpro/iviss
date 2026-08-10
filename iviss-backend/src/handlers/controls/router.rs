use crate::app_state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

/// Admin-only paged control listing.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/v1/admin/controls/paged",
        get(super::get_list_control_paged),
    )
}

/// Routes for any authenticated user (agent logging/listing controls).
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/v1/controls",
        get(super::get_list_control).post(super::create_control),
    )
}
