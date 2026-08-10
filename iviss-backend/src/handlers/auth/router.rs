use crate::app_state::AppState;
use axum::{routing::post, Router};
use std::sync::Arc;

/// Routes with no auth requirement — no layer must ever be added here.
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/auth/login", post(super::login))
        .route("/api/v1/auth/activate", post(super::activate))
        .route("/api/v1/auth/refresh", post(super::request_refresh))
        .route("/api/v1/auth/refresh/verify", post(super::verify_refresh))
        .route(
            "/api/v1/auth/request-daily-login",
            post(super::request_daily_login),
        )
        .route(
            "/api/v1/auth/verify-daily-login",
            post(super::verify_daily_login),
        )
}

/// Routes requiring only web auth (JWT) — no role restriction.
pub fn web_auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/auth/change-password", post(super::change_password))
        .route("/api/v1/auth/logout", post(super::logout))
}
