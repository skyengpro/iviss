use crate::db::DbPool;
use crate::middleware::{cors, logging};
use axum::{routing::get, Router};
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;

pub fn assembly(pool: DbPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .layer(axum::middleware::from_fn(logging::log_request))
        .layer(cors::cors_layer())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(pool)
}
