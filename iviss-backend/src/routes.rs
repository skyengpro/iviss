use crate::app_state::AppState;
use crate::db::DbPool;
use crate::handlers::{list_control::get_list_control, search_vehicle::search_vehicle};
use crate::middleware::{auth, cors};
use axum::middleware::from_fn_with_state;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;

pub fn assembly(pool: DbPool, jwt_secret: String) -> Router {
    let state = Arc::new(AppState::new(pool, jwt_secret));

    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register));

    let protected_routes = Router::new()
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
        .route("/auth/logout", post(crate::handlers::auth::logout))
        .layer(from_fn_with_state(state.clone(), auth::require_auth));

    public_routes
        .merge(protected_routes)
        .layer(cors::cors_layer())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(state)
}
