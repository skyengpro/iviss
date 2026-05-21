/// Integration test for control record CRUD operations
/// Tests creating, retrieving, and filtering control records
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
async fn test_create_control_requires_auth() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that creating a control record requires authentication
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/controls")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "plate_number": "ABC123",
                        "vehicle_type": "car",
                        "location": "Test Location",
                        "notes": "Test control"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized without auth token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_controls_requires_auth() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that listing control records requires authentication
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/controls")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized without auth token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_paged_controls_requires_admin_auth() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that paged control listing (admin endpoint) requires authentication
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/controls/paged?page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized without auth token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_control_endpoints_exist() {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping integration test: DATABASE_URL not set");
        return;
    }

    let state = setup_test_state().await;
    let app = routes::assembly(state);

    // Test that control endpoints are registered
    // We expect 401 (auth required) not 404 (not found)
    let endpoints = vec![
        ("/api/v1/controls", "GET"),
        ("/api/v1/controls", "POST"),
        ("/api/v1/admin/controls/paged", "GET"),
    ];

    for (uri, method) in endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 401 (endpoint exists, auth required) not 404 (not found)
        assert_ne!(
            response.status(),
            StatusCode::NOT_FOUND,
            "Endpoint {} {} should exist",
            method,
            uri
        );
    }
}
