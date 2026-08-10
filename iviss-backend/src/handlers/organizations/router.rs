use crate::app_state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

/// Admin-only organization management routes.
///
/// `GET /admin/organizations` lists organizations via `handlers::users` — a
/// pre-existing cross-domain handler placement (P4), not touched this
/// iteration — while `POST` creates one via this domain's own handler.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/admin/organizations",
            get(crate::handlers::users::list_organizations).post(super::create_organization),
        )
        .route(
            "/api/v1/admin/organizations/:id",
            get(super::get_organization)
                .put(super::update_organization)
                .delete(super::delete_organization),
        )
}
