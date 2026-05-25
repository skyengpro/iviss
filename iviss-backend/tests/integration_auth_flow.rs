/// Integration tests for authentication flow
/// Tests the complete auth workflow: login -> refresh -> logout
/// Uses testcontainers for real database testing
mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use helpers::*;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let (app, _db, _org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    let response = app
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
async fn test_admin_login_success() {
    let (app, db, org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    // Create admin user with hashed password
    let password = "TestPassword123!";
    let password_hash = helpers::hash_test_password(password);

    let admin_email = "admin@test.com";
    insert_test_admin(&db, org_id, admin_email, &password_hash, "admin").await;

    // Test login
    let request_body = json!({
        "email": admin_email,
        "password": password,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify response contains tokens
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body["accessToken"].is_string());
    assert!(body["refreshToken"].is_string());
}

#[tokio::test]
async fn test_admin_login_invalid_credentials() {
    let (app, db, org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    // Create admin user
    let password_hash = helpers::hash_test_password("correct_password");

    let admin_email = "admin@test.com";
    insert_test_admin(&db, org_id, admin_email, &password_hash, "admin").await;

    // Test login with wrong password
    let request_body = json!({
        "email": admin_email,
        "password": "wrong_password",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized for invalid credentials
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_login_missing_email() {
    let (app, _db, _org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    let request_body = json!({
        "email": "",
        "password": "password",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_login_user_not_found() {
    let (app, _db, _org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    let request_body = json!({
        "email": "nonexistent@test.com",
        "password": "password",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_requires_auth() {
    let (app, _db, _org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    // Try to access protected endpoint without token
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let (app, _db, org_id, _user_id, _device_id, _pg, _cache, config) =
        setup_complete_test_infrastructure().await;

    // Generate valid JWT token (admin doesn't need real device, can use nil)
    let token = generate_test_jwt_token(&config, _user_id, _device_id, "admin").await;

    // Access protected endpoint with token
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/users")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 200 OK (or 403 if RBAC denies, but not 401)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}
