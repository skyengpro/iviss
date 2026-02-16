use crate::app_state::AppState;
use crate::db::DbPool;
// use crate::middleware::{cors, logging};
use crate::handlers::{
    list_control::get_list_control,
    // pending_submission::submit_vehicle,
    search_vehicle::search_vehicle,
};
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
        .route("/controls", get(get_list_control))
        .route(
            "/vehicles/pending",
            post(crate::handlers::pending_submission::submit_vehicle),
        )
        .route("/stats", get(crate::handlers::stats::get_dashboard_stats))
        .route("/users/me", get(crate::handlers::users::get_user_profile))
        // .layer(axum::middleware::from_fn(logging::log_request))
        // .layer(cors::cors_layer())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(state)
}
