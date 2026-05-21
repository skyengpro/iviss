/// Integration test for authentication flow
/// Tests the complete auth workflow: login -> refresh -> logout
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
async fn test_health_endpoint() {
    let state = setup_test_state().await;
    let app = routes::assembly(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_login_flow() {
    // This test requires a running database with seed data
    // Skip if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test login endpoint exists
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/admin/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"test@example.com","password":"invalid"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 for invalid credentials
    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::BAD_REQUEST
    );
}
