use axum::{
    routing::{get, post},
    Router,
};
use crate::handlers::{auth, organizations};
use crate::db::DbPools;
use crate::middleware::{logging, cors};

pub fn assembly(pools: DbPools) -> Router {
    Router::new()
        .route("/auth/login", post(auth::login))
        .route("/organizations", get(organizations::list))
        .layer(axum::middleware::from_fn(logging::log_request))
        .layer(cors::cors_layer())
        .with_state(pools)
}
