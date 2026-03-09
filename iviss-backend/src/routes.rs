use crate::app_state::AppState;
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

pub fn assembly(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/api/v1/scan/plate",
            post(crate::handlers::scan::scan_plate),
        )
        .route(
            "/api/v1/photo/plate",
            post(crate::handlers::photo::photo_plate),
        )
        .route("/vehicles/search", post(search_vehicle))
        .route("/api/v1/vehicles/search", post(search_vehicle))
        .route(
            "/controls",
            get(get_list_control).post(crate::handlers::list_control::create_control),
        )
        .route(
            "/vehicles/pending",
            post(crate::handlers::pending_submission::submit_vehicle),
        )
        .route(
            "/api/v1/vehicles/pending",
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
        .route(
            "/users/location",
            post(crate::handlers::users::update_location),
        )
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .route("/auth/logout", post(crate::handlers::auth::logout))
        .route(
            "/auth/send-activation",
            post(crate::handlers::auth::send_activation),
        )
        .route("/auth/activate", post(crate::handlers::auth::activate))
        .route(
            "/admin/users",
            get(crate::handlers::user_management::list_users)
                .post(crate::handlers::user_management::provision_user),
        )
        .route(
            "/admin/users/:id",
            get(crate::handlers::user_management::get_user)
                .put(crate::handlers::user_management::update_user)
                .delete(crate::handlers::user_management::delete_user),
        )
        .route(
            "/admin/organizations",
            get(crate::handlers::user_management::list_organizations),
        )
        // .layer(axum::middleware::from_fn(logging::log_request))
        .layer(cors::cors_layer())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors::cors_layer())
        .with_state(state)
}
