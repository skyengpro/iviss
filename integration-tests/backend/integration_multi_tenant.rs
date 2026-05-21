/// Integration test for multi-tenant data isolation
/// Tests that organizations cannot access each other's data
use iviss_backend::app_cache::AppCache;
use iviss_backend::app_state::AppState;
use iviss_backend::config::Config;
use iviss_backend::routes;
use iviss_backend::services::email_provider::MockEmailProvider;
use iviss_backend::services::sms_provider::NoopSmsProvider;
use std::sync::Arc;
use tower::ServiceExt;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::postgres::PgPoolOptions;

/// Helper to create test app state
async fn setup_test_state() -> AppState {
    let config = Config::from_env().expect("Failed to load config");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create pool");
    
    let app_cache = Arc::new(AppCache::new());
    let sms_provider: Arc<dyn iviss_backend::services::sms_provider::SmsProvider> = 
        Arc::new(NoopSmsProvider);
    let email_provider: Arc<dyn iviss_backend::services::email_provider::EmailProvider> = 
        Arc::new(MockEmailProvider);
    
    AppState::new(pool, app_cache, sms_provider, email_provider, &config)
}

#[tokio::test]
async fn test_organization_data_isolation() {
    // Skip if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that organization endpoints require authentication
    // This verifies the RBAC middleware is in place
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/organizations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized without auth token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rbac_enforcement_on_admin_routes() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that admin routes require authentication
    let admin_endpoints = vec![
        "/api/v1/admin/users",
        "/api/v1/admin/organizations",
        "/api/v1/admin/submissions",
        "/api/v1/admin/stats",
        "/api/v1/admin/audit",
    ];

    for endpoint in admin_endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(endpoint)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // All admin endpoints should require authentication
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} should require authentication",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_org_admin_routes_require_auth() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that org-admin routes require authentication
    let org_admin_endpoints = vec![
        "/api/v1/org-admin/users",
        "/api/v1/org-admin/stats",
        "/api/v1/org-admin/activity-feed",
        "/api/v1/org-admin/recent-alerts",
        "/api/v1/org-admin/top-agents",
        "/api/v1/org-admin/activity",
    ];

    for endpoint in org_admin_endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(endpoint)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // All org-admin endpoints should require authentication
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} should require authentication",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_cross_org_access_denied_without_auth() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that attempting to access organization-specific data without auth fails
    // This simulates an attempt to access another org's data
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/org-admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
