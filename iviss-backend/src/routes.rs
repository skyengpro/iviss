use crate::app_state::AppState;
use crate::handlers;
use crate::middleware::{auth, cors, metrics, rbac};
use axum::http::HeaderValue;
use axum::middleware::from_fn_with_state;
use axum::{routing::get, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

pub fn assembly(state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/api/v1/health", get(handlers::health::health_check))
        .merge(handlers::auth::router::public_routes());

    // Admin routes require both web auth (JWT) and admin role check
    let admin_routes = Router::new()
        .merge(handlers::submissions::router::admin_routes())
        .merge(handlers::users::router::admin_routes())
        .merge(handlers::organizations::router::admin_routes())
        .merge(handlers::controls::router::admin_routes())
        .merge(handlers::audit::router::admin_routes())
        .merge(handlers::stats::router::admin_routes())
        .layer(from_fn_with_state(state.clone(), rbac::require_admin))
        .layer(from_fn_with_state(state.clone(), rbac::require_auth_web));

    // Org-admin routes — scoped to org_admin users with a valid organization_id
    let org_admin_routes = Router::new()
        .merge(handlers::users::router::org_admin_routes())
        .merge(handlers::stats::router::org_admin_routes())
        .layer(from_fn_with_state(state.clone(), rbac::require_org_admin))
        .layer(from_fn_with_state(state.clone(), rbac::require_auth_web));

    // Web-authenticated routes - accessible to admin, manager, org_admin
    let web_auth_routes = Router::new()
        .merge(handlers::auth::router::web_auth_routes())
        .layer(from_fn_with_state(state.clone(), rbac::require_auth_web));

    // Routes for any authenticated user (agent, manager, admin, org_admin)
    let protected_routes = Router::new()
        .merge(handlers::ocr::router::protected_routes())
        .merge(handlers::vehicles::router::protected_routes())
        .merge(handlers::controls::router::protected_routes())
        .merge(handlers::submissions::router::protected_routes())
        .merge(handlers::stats::router::protected_routes())
        .merge(handlers::users::router::protected_routes())
        .layer(from_fn_with_state(state.clone(), auth::require_auth));

    public_routes
        .merge(admin_routes)
        .merge(org_admin_routes)
        .merge(web_auth_routes)
        .merge(protected_routes)
        .layer(axum::middleware::from_fn(metrics::track_metrics))
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        ))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors::cors_layer(&state.cors_allowed_origins))
        .with_state(state)
}

/// Internal metrics server — served on a separate port (9091) so that
/// /metrics is only accessible from within the cluster (Prometheus
/// ServiceMonitor), NOT through the public ingress.
pub fn metrics_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/metrics", get(handlers::health::metrics_export))
        .with_state(state)
}
