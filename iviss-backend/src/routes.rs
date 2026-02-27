use crate::app_state::AppState;
use crate::db::DbPool;
use crate::handlers::{
    list_control::get_list_control,
    // pending_submission::submit_vehicle,
    search_vehicle::search_vehicle,
};
use crate::middleware::cors;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;

pub fn assembly(pool: DbPool) -> Router {
    let state = Arc::new(AppState::new(pool));
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/vehicles/search", post(search_vehicle))
        .route(
            "/controls",
            get(get_list_control).post(crate::handlers::list_control::create_control),
        )
        .route(
            "/vehicles/pending",
            post(crate::handlers::pending_submission::submit_vehicle),
        )
        .route(
            "/admin/submissions",
            get(crate::handlers::pending_submission::list_pending_submissions),
        )
        .route(
            "/admin/submissions/:id",
            get(crate::handlers::pending_submission::get_pending_submission),
        )
        .route("/stats", get(crate::handlers::stats::get_dashboard_stats))
        .route("/users/me", get(crate::handlers::users::get_user_profile))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .route("/auth/logout", post(crate::handlers::auth::logout))
        .route(
            "/admin/users",
            get(crate::handlers::admin::list_users).post(crate::handlers::admin::provision_user),
        )
        .route(
            "/admin/users/:id",
            get(crate::handlers::admin::get_user)
                .put(crate::handlers::admin::update_user)
                .delete(crate::handlers::admin::delete_user),
        )
        .route(
            "/admin/organizations",
            get(crate::handlers::admin::list_organizations),
        )
        // .layer(axum::middleware::from_fn(logging::log_request))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors::cors_layer())
        .with_state(state)
}
