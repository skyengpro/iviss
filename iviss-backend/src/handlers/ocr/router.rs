use crate::app_state::AppState;
use axum::{routing::post, Router};
use std::sync::Arc;

/// Routes for any authenticated user (agent scanning a plate).
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/scan/plate", post(super::scan_plate))
        .route("/api/v1/photo/plate", post(super::photo_plate))
}
