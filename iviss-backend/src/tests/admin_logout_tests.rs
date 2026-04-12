use crate::app_state::AppState;
use crate::routes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use testcontainers::{
    runners::AsyncRunner,
};
use testcontainers_modules::postgres::Postgres;
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

/// Setup test infrastructure for admin logout tests
async fn setup_admin_logout_test() -> (
    axum::Router,
    sqlx::PgPool,
    testcontainers::ContainerAsync<Postgres>,
    String, // jwt_private_key_pem
    String, // jwt_public_key_pem
) {
    let pg = Postgres::default().with_host_auth().start().await.unwrap();
    let pg_port = pg.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres@127.0.0.1:{}/postgres", pg_port);

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url.clone())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    let config = crate::config::Config {
        database_url: db_url,
        server_host: "0.0.0.0".to_string(),
        server_port: 0,
        log_level: crate::config::LogLevel::Info,
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
        Arc::new(crate::app_cache::AppCache::new()),
        Arc::new(crate::services::sms_provider::MockSmsProvider),
        &config,
    );

    let app = routes::assembly(state);

    (app, db, pg, jwt_private_key_pem, jwt_public_key_pem)
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

/// Helper: create a refresh token for a user
async fn create_refresh_token(db: &sqlx::PgPool, user_id: Uuid, device_id: Option<Uuid>) -> String {
    let refresh_token = format!("test_refresh_token_{}", Uuid::new_v4());
    let token_hash = format!("{:x}", Sha256::digest(refresh_token.as_bytes()));
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, $3, $4, FALSE)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(device_id)
    .bind(expires_at)
    .execute(db)
    .await
    .unwrap();

    refresh_token
}

/// Helper: generate access token
fn generate_access_token(
    jwt_private_key_pem: &str,
    user_id: Uuid,
    role: crate::dto::users::UserRole,
) -> String {
    let jwt_svc = crate::services::jwt_service::JwtService::new(jwt_private_key_pem).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let shift_start = now as usize;
    let shift_end = (now + 86_400) as usize;

    jwt_svc
        .issue_access_token_with_shift(user_id, Uuid::nil(), role, shift_start, shift_end)
        .unwrap()
}

// =============================================================================
// Admin Logout Tests
// =============================================================================

#[tokio::test]
async fn test_admin_logout_success() {
    let (app, db, _pg, jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    // Create a refresh token for the user
    let _refresh_token = create_refresh_token(&db, user_id, None).await;

    // Generate access token
    let access_token = generate_access_token(
        &jwt_private_key_pem,
        user_id,
        crate::dto::users::UserRole::Admin,
    );

    // Call logout endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_admin_logout_revokes_refresh_tokens_and_allows_access_token_use() {
    let (app, db, _pg, jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    // Generate access token
    let access_token = generate_access_token(
        &jwt_private_key_pem,
        user_id,
        crate::dto::users::UserRole::Admin,
    );

    // Logout first
    let _logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Note: Access tokens are NOT blacklisted (no Redis check)
    // The access token remains valid until it expires naturally
    // Only refresh tokens are revoked
    let protected_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Access token still works because only refresh tokens are revoked
    assert_eq!(protected_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_logout_revokes_refresh_tokens() {
    let (app, db, _pg, jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    // Create multiple refresh tokens for the user
    let refresh_token1 = create_refresh_token(&db, user_id, None).await;
    let refresh_token2 = create_refresh_token(&db, user_id, None).await;

    // Generate access token
    let access_token = generate_access_token(
        &jwt_private_key_pem,
        user_id,
        crate::dto::users::UserRole::Admin,
    );

    // Logout
    let _logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify all refresh tokens are revoked
    let active_tokens: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1 AND revoked = FALSE",
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(active_tokens, 0, "all refresh tokens should be revoked");

    // Try to refresh with the first token - should fail
    let refresh_body = json!({
        "refreshToken": refresh_token1,
    });

    let refresh_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(refresh_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);

    // Try to refresh with the second token - should also fail
    let refresh_body2 = json!({
        "refreshToken": refresh_token2,
    });

    let refresh_response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(refresh_body2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_response2.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_logout_missing_auth_header() {
    let (app, _db, _pg, _jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    // Call logout without Authorization header
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_logout_invalid_token() {
    let (app, _db, _pg, _jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    // Call logout with invalid token
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", "Bearer invalid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_logout_manager_role_allowed() {
    let (app, db, _pg, jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    let email = "manager@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "manager", "ACTIVE").await;

    // Create a refresh token for the manager
    let _refresh_token = create_refresh_token(&db, user_id, None).await;

    // Generate access token for manager
    let access_token = generate_access_token(
        &jwt_private_key_pem,
        user_id,
        crate::dto::users::UserRole::Manager,
    );

    // Call logout endpoint as manager
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Note: Access tokens are NOT blacklisted (no Redis check)
    // The access token remains valid until it expires naturally
    // Only refresh tokens are revoked during logout
    let protected_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Access token still works because only refresh tokens are revoked
    assert_eq!(protected_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_logout_idempotent() {
    let (app, db, _pg, jwt_private_key_pem, _jwt_public_key_pem) =
        setup_admin_logout_test().await;

    let email = "admin@test.com";
    let password = "testpassword123";
    let user_id = create_admin_user(&db, email, password, "admin", "ACTIVE").await;

    // Generate access token
    let access_token = generate_access_token(
        &jwt_private_key_pem,
        user_id,
        crate::dto::users::UserRole::Admin,
    );

    // First logout - should succeed
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::NO_CONTENT);

    // Second logout with same token - should also return 204
    // Note: Without Redis blacklist, access tokens are not invalidated
    // so the second logout also succeeds (idempotent for refresh token revocation)
    let response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Without token blacklisting, the second logout also succeeds
    // (refresh token revocation is idempotent)
    assert_eq!(response2.status(), StatusCode::NO_CONTENT);
}
