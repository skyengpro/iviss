use crate::app_state::AppState;
use axum::{routing::post, Router};
use std::sync::Arc;

/// Routes for any authenticated user (agent searching a plate).
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/vehicles/search", post(super::search_vehicle_v1))
}
