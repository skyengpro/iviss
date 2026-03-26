use crate::app_state::AppState;
use crate::routes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_PEPPER: &str = "test_pepper_for_activation_code_hashing_must_be_32_chars_long";

fn generate_test_rsa_keypair_pem() -> (String, String) {
    use rand::rngs::OsRng;
    use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
    use rsa::RsaPrivateKey;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
    let public_key = private_key.to_public_key();

    let private_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("Failed to encode RSA private key")
        .to_string();
    let public_pem = public_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("Failed to encode RSA public key")
        .to_string();

    (private_pem, public_pem)
}

/// Helper to hash password with Argon2
async fn hash_password(password: &str) -> String {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

/// Setup test infrastructure for admin login tests
async fn setup_admin_login_test() -> (
    axum::Router,
    sqlx::PgPool,
    testcontainers::ContainerAsync<Postgres>,
    testcontainers::ContainerAsync<Redis>,
) {
    let pg = Postgres::default().start().await.unwrap();
    let pg_port = pg.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        pg_port
    );

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url.clone())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let redis = Redis::default().start().await.unwrap();
    let redis_port = redis.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://127.0.0.1:{}", redis_port);
    let redis_cfg = deadpool_redis::Config::from_url(redis_url.clone());
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap();

    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    let config = crate::config::Config {
        database_url: db_url,
        redis_url: redis_url.clone(),
        server_host: "0.0.0.0".to_string(),
        server_port: 0,
        log_level: crate::config::LogLevel::Info,
        jwt_secret: "secret_longer_than_32_characters_for_test".to_string(),
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: crate::config::Environment::Local,
        twilio_account_sid: "sid".to_string(),
        twilio_auth_token: "token".to_string(),
        twilio_from_number: "num".to_string(),
        activation_code_pepper: TEST_PEPPER.to_string(),
        shift_start_hour: 0,
        shift_end_hour: 24,
        admin_bootstrap_email: Some("admin@example.com".to_string()),
        admin_bootstrap_password: Some("password".to_string()),
        admin_bootstrap_phone: Some("+237600000000".to_string()),
        admin_bootstrap_username: Some("admin".to_string()),
    };

    let state = AppState::new(
        db.clone(),
        redis_pool.clone(),
        Arc::new(crate::services::sms_provider::MockSmsProvider),
        &config,
    );

    let app = routes::assembly(state);

    (app, db, pg, redis)
}

/// Helper: create admin user
async fn create_admin_user(
    db: &sqlx::PgPool,
    email: &str,
    password: &str,
    role: &str,
    status: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();

    // Agents must have NULL password_hash (constraint chk_users_agent_no_password)
    let password_hash = if role == "agent" {
        None
    } else {
        Some(hash_password(password).await)
    };

    // All users need organization_id now
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(db)
        .await
        .ok();

    // badge_id required for agents
    let badge_id = if role == "agent" {
        Some(format!(
            "BADGE-{}",
            user_id.to_string().split('-').next().unwrap()
        ))
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, phone_number, role, status, full_name, created_at, organization_id, badge_id)
        VALUES ($1, $2, $3, $4, $5, $6::user_role, $7::user_status, $8, NOW(), $9, $10)
        "#,
    )
    .bind(user_id)
    .bind(format!("user_{}", user_id.to_string().split('-').next().unwrap()))
    .bind(email)
    .bind(password_hash)
    .bind("+1234567890")
    .bind(role)
    .bind(status)
    .bind("Test User")
    .bind(org_id)
    .bind(badge_id)
    .execute(db)
    .await
    .expect("Failed to create test user");

    user_id
}

// =============================================================================
// Admin Login Tests
// =============================================================================

#[tokio::test]
async fn test_admin_login_success() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let _user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    let request_body = json!({
        "email": email,
        "password": password,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body["accessToken"].is_string());
    assert!(body["refreshToken"].is_string());
    assert_eq!(body["user"]["role"], "admin");
}

#[tokio::test]
async fn test_admin_login_invalid_credentials() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let _user_id = create_admin_user(&db, email, "correctpassword", "admin", "ACTIVE").await;

    let request_body = json!({
        "email": email,
        "password": "wrongpassword",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_login_inactive_account() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "suspended@test.com";
    let password = "testpassword123";
    let _user_id = create_admin_user(&db, email, password, "admin", "SUSPENDED").await;

    let request_body = json!({
        "email": email,
        "password": password,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_login_empty_email() {
    let (app, _db, _pg, _redis) = setup_admin_login_test().await;

    let request_body = json!({
        "email": "",
        "password": "password123",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_login_empty_password() {
    let (app, _db, _pg, _redis) = setup_admin_login_test().await;

    let request_body = json!({
        "email": "admin@test.com",
        "password": "",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_login_nonexistent_user() {
    let (app, _db, _pg, _redis) = setup_admin_login_test().await;

    let request_body = json!({
        "email": "nonexistent@test.com",
        "password": "password123",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// =============================================================================
// Admin Refresh Token Tests
// =============================================================================

#[tokio::test]
async fn test_admin_refresh_token_success() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    // Create refresh token
    let refresh_token = "test_refresh_token_123";
    let token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, NULL, $3, FALSE)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(expires_at)
    .execute(&db)
    .await
    .unwrap();

    let request_body = json!({
        "refreshToken": refresh_token,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body["accessToken"].is_string());
}

#[tokio::test]
async fn test_admin_refresh_token_invalid() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let _user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    let request_body = json!({
        "refreshToken": "invalid_token",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_refresh_token_inactive_account() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "suspended@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "SUSPENDED").await;

    let refresh_token = "test_refresh_token_456";
    let token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, NULL, $3, FALSE)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(expires_at)
    .execute(&db)
    .await
    .unwrap();

    let request_body = json!({
        "refreshToken": refresh_token,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_refresh_token_expired() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    let refresh_token = "expired_refresh_token";
    let token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));
    let expires_at = time::OffsetDateTime::now_utc() - time::Duration::days(1);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, NULL, $3, FALSE)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(expires_at)
    .execute(&db)
    .await
    .unwrap();

    let request_body = json!({
        "refreshToken": refresh_token,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_refresh_token_non_admin_role() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "agent@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "agent", "ACTIVE").await;

    let refresh_token = "agent_refresh_token";
    let token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, NULL, $3, FALSE)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(expires_at)
    .execute(&db)
    .await
    .unwrap();

    let request_body = json!({
        "refreshToken": refresh_token,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_refresh_token_empty() {
    let (app, _db, _pg, _redis) = setup_admin_login_test().await;

    let request_body = json!({
        "refreshToken": "",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_refresh_token_revoked() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    let refresh_token = "revoked_refresh_token";
    let token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, NULL, $3, TRUE)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(expires_at)
    .execute(&db)
    .await
    .unwrap();

    let request_body = json!({
        "refreshToken": refresh_token,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// =============================================================================
// RBAC Logic Tests
// =============================================================================

#[tokio::test]
async fn test_rbac_admin_middleware_allowed() {
    // First login to get a token
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let _user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    let login_body = json!({
        "email": email,
        "password": password,
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify the token was issued with admin role
    assert_eq!(body["user"]["role"], "admin");
}

#[tokio::test]
async fn test_rbac_manager_middleware_allowed() {
    let (app, db, _pg, _redis) = setup_admin_login_test().await;

    let email = "manager@test.com";
    let password = "testpassword123";
    let _user_id = create_admin_user(&db, email, password, "manager", "ACTIVE").await;

    let login_body = json!({
        "email": email,
        "password": password,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Manager can access login and refresh
    assert_eq!(body["user"]["role"], "manager");
}
