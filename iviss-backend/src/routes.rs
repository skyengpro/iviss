use crate::app_state::AppState;
use crate::handlers::list_control::{get_list_control, get_list_control_paged};
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
        .route("/api/v1/health", get(crate::handlers::health::health_check))
        .route("/api/v1/auth/login", post(crate::handlers::auth::login))
        .route(
            "/api/v1/auth/activate",
            post(crate::handlers::auth::activate),
        )
        .route(
            "/api/v1/auth/refresh",
            post(crate::handlers::auth::request_refresh),
        )
        .route(
            "/api/v1/auth/refresh/verify",
            post(crate::handlers::auth::verify_refresh),
        )
        .route(
            "/api/v1/auth/request-daily-login",
            post(crate::handlers::auth::request_daily_login).layer(from_fn_with_state(
                state.clone(),
                crate::middleware::agent_work_scope::require_shift_hours,
            )),
        )
        .route(
            "/api/v1/auth/verify-daily-login",
            post(crate::handlers::auth::verify_daily_login),
        );

    // Admin routes require both web auth (JWT) and admin role check
    let admin_routes = Router::new()
        .route(
            "/api/v1/admin/submissions",
            get(crate::handlers::pending_submission::list_pending_submissions),
        )
        .route(
            "/api/v1/admin/submissions/:id",
            get(crate::handlers::pending_submission::get_pending_submission),
        )
        .route(
            "/api/v1/admin/users",
            get(crate::handlers::user_management::list_users)
                .post(crate::handlers::user_management::provision_user),
        )
        .route(
            "/api/v1/admin/users/:id",
            get(crate::handlers::user_management::get_user)
                .put(crate::handlers::user_management::update_user)
                .delete(crate::handlers::user_management::delete_user),
        )
        .route(
            "/api/v1/admin/organizations",
            get(crate::handlers::user_management::list_organizations),
        )
        .route(
            "/api/v1/admin/terminate-session",
            post(crate::handlers::user_management::terminate_session),
        )
        .route(
            "/api/v1/admin/restart-session",
            post(crate::handlers::user_management::restart_session),
        )
        .route(
            "/api/v1/admin/resend-activation-code",
            post(crate::handlers::user_management::resend_activation_code),
        )
        .route("/api/v1/admin/controls/paged", get(get_list_control_paged))
        .route(
            "/api/v1/admin/stats",
            get(crate::handlers::stats::get_dashboard_stats),
        )
        .route(
            "/api/v1/admin/audit-logs",
            get(crate::handlers::audit_log::list_audit_logs),
        )
        .route(
            "/api/v1/admin/audit-logs/export",
            get(crate::handlers::audit_log::export_audit_logs),
        )
        .layer(from_fn_with_state(state.clone(), rbac::require_admin))
        .layer(from_fn_with_state(state.clone(), rbac::require_auth_web));

    let protected_routes = Router::new()
        .route(
            "/api/v1/scan/plate",
            post(crate::handlers::scan::scan_plate),
        )
        .route(
            "/api/v1/photo/plate",
            post(crate::handlers::photo::photo_plate),
        )
        .route(
            "/api/v1/vehicles/search",
            post(crate::handlers::search_vehicle::search_vehicle_v1),
        )
        .route(
            "/api/v1/controls",
            get(get_list_control).post(crate::handlers::list_control::create_control),
        )
        .route(
            "/api/v1/vehicles/pending",
            post(crate::handlers::pending_submission::submit_vehicle_v1),
        )
        .route(
            "/api/v1/stats/activity",
            get(crate::handlers::stats::get_control_activity),
        )
        .route(
            "/api/v1/stats/top-agents",
            get(crate::handlers::stats::get_top_agents),
        )
        .route(
            "/api/v1/stats/activity-feed",
            get(crate::handlers::stats::get_activity_feed),
        )
        .route(
            "/api/v1/stats/recent-alerts",
            get(crate::handlers::stats::get_recent_alerts),
        )
        .route(
            "/api/v1/users/me",
            get(crate::handlers::users::get_user_profile),
        )
        .route(
            "/api/v1/users/location",
            post(crate::handlers::users::update_location),
        )
        // TODO: Enable once logout is fully implemented
        // .route("/auth/logout", post(crate::handlers::auth::logout))
        .layer(from_fn_with_state(state.clone(), auth::require_auth));

    public_routes
        .merge(admin_routes)
        .merge(protected_routes)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors::cors_layer())
        .with_state(state)
}
