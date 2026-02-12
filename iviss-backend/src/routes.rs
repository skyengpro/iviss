// use crate::db::DbPool;
// use crate::middleware::{cors, logging};
use crate::handlers::{
    list_control::get_list_control,
    // pending_submission::submit_vehicle,
    search_vehicle::search_vehicle,
};
use axum::{routing::get, routing::post, Router};

pub fn assembly(/* pool: DbPool */) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/vehicle/search", post(search_vehicle))
        .route("/controls", get(get_list_control))
    // .route("/vehicles/pending", post(submit_vehicle))
    // .layer(axum::middleware::from_fn(logging::log_request))
    // .layer(cors::cors_layer())
    // .with_state(pool)
}
