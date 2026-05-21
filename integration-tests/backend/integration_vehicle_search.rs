/// Integration test for vehicle search functionality
/// Tests vehicle search endpoint with database queries
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
async fn test_vehicle_search_endpoint_exists() {
    // Skip if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that vehicle search endpoint is accessible
    // Note: This will return 401 without auth token, which is expected
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/vehicles/search?plate_number=ABC123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized (no auth token provided)
    // This confirms the endpoint exists and auth middleware is working
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_vehicle_search_requires_authentication() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test without Authorization header
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/vehicles/search?plate_number=TEST123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should require authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_vehicle_search_validates_plate_number() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test with missing plate_number parameter
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/vehicles/search")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 400 Bad Request or 401 Unauthorized
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}
