use crate::app_state::AppState;
use crate::handlers::{list_control::get_list_control, search_vehicle::search_vehicle};
use crate::middleware::{auth, cors, rbac};
use axum::middleware::from_fn_with_state;
use axum::{routing::get, routing::post, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;

pub fn assembly(state: AppState) -> Router {
    let state = Arc::new(state);

    let public_routes = Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .route("/auth/login", post(crate::handlers::auth::login))
        .route("/auth/register", post(crate::handlers::auth::register))
        .route("/auth/activate", post(crate::handlers::auth::activate))
        .route(
            "/auth/refresh",
            post(crate::handlers::auth::request_refresh),
        )
        .route(
            "/auth/refresh/verify",
            post(crate::handlers::auth::verify_refresh),
        )
        .route(
            "/auth/request-daily-login",
            post(crate::handlers::auth::request_daily_login).layer(from_fn_with_state(
                state.clone(),
                crate::middleware::agent_work_scope::require_shift_hours,
            )),
        )
        .route(
            "/auth/verify-daily-login",
            post(crate::handlers::auth::verify_daily_login),
        );

    // Admin routes require both web auth (JWT) and admin role check
    let admin_routes = Router::new()
        .route(
            "/admin/submissions",
            get(crate::handlers::pending_submission::list_pending_submissions),
        )
        .route(
            "/admin/submissions/:id",
            get(crate::handlers::pending_submission::get_pending_submission),
        )
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
        .route(
            "/admin/terminate-session",
            post(crate::handlers::user_management::terminate_session),
        )
        .route(
            "/admin/devices/{id}/suspend",
            post(crate::handlers::device_management::suspend_device),
        )
        .route(
            "/admin/devices/{id}/unsuspend",
            post(crate::handlers::device_management::unsuspend_device),
        )
        .route(
            "/admin/resend-activation-code",
            post(crate::handlers::user_management::resend_activation_code),
        )
        .route("/stats", get(crate::handlers::stats::get_dashboard_stats))
        .layer(from_fn_with_state(state.clone(), rbac::require_auth_web))
        .layer(from_fn_with_state(state.clone(), rbac::require_admin));

    let protected_routes = Router::new()
        .route(
            "/api/v1/scan/plate",
            post(crate::handlers::scan::scan_plate),
        )
        .route(
            "/api/v1/photo/plate",
            post(crate::handlers::photo::photo_plate),
        )
        .route("/vehicles/search", post(search_vehicle))
        .route(
            "/api/v1/vehicles/search",
            post(crate::handlers::search_vehicle::search_vehicle_v1),
        )
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
            post(crate::handlers::pending_submission::submit_vehicle_v1),
        )
        .route("/users/me", get(crate::handlers::users::get_user_profile))
        .route(
            "/users/location",
            post(crate::handlers::users::update_location),
        )
        .route("/auth/logout", post(crate::handlers::auth::logout))
        .layer(from_fn_with_state(state.clone(), auth::require_auth));

    public_routes
        .merge(admin_routes)
        .merge(protected_routes)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors::cors_layer())
        .with_state(state)
}
