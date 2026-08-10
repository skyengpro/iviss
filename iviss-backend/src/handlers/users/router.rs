use crate::app_state::AppState;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;

/// Admin-only user & session management routes.
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/admin/users",
            get(super::list_users).post(super::provision_user),
        )
        .route(
            "/api/v1/admin/users/:id",
            get(super::get_user)
                .put(super::update_user)
                .delete(super::delete_user),
        )
        .route(
            "/api/v1/admin/terminate-session",
            post(super::terminate_session),
        )
        .route(
            "/api/v1/admin/restart-session",
            post(super::restart_session),
        )
        .route(
            "/api/v1/admin/resend-activation-code",
            post(super::resend_activation_code),
        )
        .route(
            "/api/v1/admin/resend-org-admin-password",
            post(super::resend_org_admin_password),
        )
}

/// Org-admin scoped user management routes.
pub fn org_admin_routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/v1/org-admin/users",
        get(super::list_org_users).post(super::provision_org_user),
    )
}

/// Routes for any authenticated user (agent, manager, admin, org_admin).
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/users/me", get(super::get_user_profile))
        .route("/api/v1/users/location", post(super::update_location))
}
