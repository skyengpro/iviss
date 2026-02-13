use crate::db::DbPool;
use crate::app_state::AppState;
// use crate::middleware::{cors, logging};
use crate::handlers::{
    list_control::get_list_control,
    // pending_submission::submit_vehicle,
    search_vehicle::search_vehicle,
};
use axum::{routing::get, routing::post, Router};
use std::time::Duration;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;

pub fn assembly(pool: DbPool) -> Router {
    let state = Arc::new(AppState::new(pool));
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/vehicle/search", post(search_vehicle))
        .route("/controls", get(get_list_control))
    // .route("/vehicles/pending", post(submit_vehicle))
    // .layer(axum::middleware::from_fn(logging::log_request))
    // .layer(cors::cors_layer())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .with_state(state)
}
