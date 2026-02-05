use crate::db::DbPool;
use crate::middleware::{cors, logging};
use axum::{routing::get, Router};

pub fn assembly(pool: DbPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .layer(axum::middleware::from_fn(logging::log_request))
        .layer(cors::cors_layer())
        .with_state(pool)
}
