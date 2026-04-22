use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use base64::Engine;
use hmac::Mac;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    app_cache::AppCache,
    app_state::AppState,
    config::{Config, Environment, LogLevel},
    dto::users::{UserRole, UserStatus},
    routes,
    services::jwt_service::JwtService,
    services::otp_service::OTP_TTL_SECS,
};

use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

const TEST_PEPPER: &str = "test_pepper";

/// Helper: hash OTP code using the same method as OtpService
fn hash_otp_code(pepper: &str, code: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts any key size");
    mac.update(code.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

/// Helper: store OTP directly in Moka cache for testing
async fn store_test_otp(
    cache: &AppCache,
    user_id: Uuid,
    code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let code_hash = hash_otp_code(TEST_PEPPER, code);
    let _entry = serde_json::json!({
        "code_hash": code_hash,
        "attempts": 0,
        "expires_at": (std::time::SystemTime::now() + std::time::Duration::from_secs(OTP_TTL_SECS))
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });

    cache
        .otp_store
        .insert(
            user_id,
            crate::app_cache::OtpEntry {
                code_hash,
                attempts: 0,
                expires_at: std::time::Instant::now()
                    + std::time::Duration::from_secs(OTP_TTL_SECS),
            },
        )
        .await;

    Ok(())
}

/// Generate a valid EC public key base64 encoded
/// The activate endpoint just checks if it's valid base64, not the actual key contents
fn generate_test_public_key_base64() -> String {
    // Use a 32-byte random key encoded in base64
    // This is valid base64 but doesn't need to be a real EC key
    // The endpoint only validates base64 encoding, not key validity
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Infrastructure Setup
// ─────────────────────────────────────────────────────────────────────────────

// Use existing test fixtures for RSA keys
const TEST_PRIVATE_KEY: &str = include_str!("fixtures/test_private_key.pem");
const TEST_PUBLIC_KEY: &str = include_str!("fixtures/test_public_key.pem");

fn generate_test_rsa_keypair_pem() -> (String, String) {
    (TEST_PRIVATE_KEY.to_string(), TEST_PUBLIC_KEY.to_string())
}

/// Helper: set up test app with PostgreSQL and Moka cache
async fn setup_test_app() -> (
    PgPool,
    std::sync::Arc<crate::app_cache::AppCache>,
    Router,
    String, // jwt_private_key_pem
    String, // jwt_public_key_pem
    testcontainers::ContainerAsync<Postgres>,
) {
    // Start PostgreSQL container
    let postgres = Postgres::default().with_host_auth().start().await.unwrap();
    let pg_host = postgres.get_host().await.unwrap();
    let pg_port = postgres.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres@{pg_host}:{pg_port}/postgres");

    // Create database pool
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    // Generate test keys
    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    // Create test config
    let config = Config {
        database_url: db_url,
        server_host: "0.0.0.0".to_string(),
        server_port: 8080,
        log_level: LogLevel::Info,
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: Environment::Local,
        sms_credentials: crate::config::SmsProviderCredentials::Mock,
        email_credentials: crate::config::EmailProviderCredentials::Mock,
        activation_code_pepper: TEST_PEPPER.to_string(),
        admin_bootstrap_email: None,
        admin_bootstrap_password: None,
        admin_bootstrap_phone: None,
        admin_bootstrap_username: None,
    };

    // Create Moka cache
    let cache = std::sync::Arc::new(crate::app_cache::AppCache::new());

    // Create app state with mock SMS provider
    let sms_provider: Arc<dyn crate::services::sms_provider::SmsProvider> =
        Arc::new(crate::services::sms_provider::MockSmsProvider);
    let email_provider: Arc<dyn crate::services::email_provider::EmailProvider> =
        Arc::new(crate::services::email_provider::MockEmailProvider);
    let state = AppState::new(
        db.clone(),
        cache.clone(),
        sms_provider,
        email_provider,
        &config,
    );

    // Create router
    let app = routes::assembly(state);

    (
        db,
        cache.clone(),
        app,
        jwt_private_key_pem,
        jwt_public_key_pem,
        postgres,
    )
}

async fn create_test_organization(db: &PgPool) -> Uuid {
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Organization")
        .bind("police")
        .execute(db)
        .await
        .unwrap();
    org_id
}

async fn create_test_user(db: &PgPool, org_id: Uuid, role: UserRole, status: UserStatus) -> Uuid {
    let user_id = Uuid::new_v4();
    let role_str = match role {
        UserRole::Admin => "admin",
        UserRole::Manager => "manager",
        UserRole::Agent => "agent",
        UserRole::OrgAdmin => "org_admin",
    };
    let status_str = match status {
        UserStatus::Active => "ACTIVE",
        UserStatus::Suspended => "SUSPENDED",
        UserStatus::PendingActivation => "PENDING_ACTIVATION",
    };

    // Agents must have NULL password_hash
    let password_hash = if role == UserRole::Agent {
        None
    } else {
        Some("$argon2id$v=19$m=65536,t=3,p=4$c2FsdHNhbHQ$hash".to_string())
    };

    // badge_id required for agents
    let badge_id = if role == UserRole::Agent {
        Some(format!(
            "BADGE-{}",
            user_id.to_string().split('-').next().unwrap()
        ))
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO users (
            id, username, email, password_hash, phone_number, 
            role, status, full_name, created_at, organization_id, badge_id
        )
        VALUES ($1, $2, $3, $4, $5, $6::user_role, $7::user_status, $8, NOW(), $9, $10)
        "#,
    )
    .bind(user_id)
    .bind(format!(
        "user_{}",
        user_id.to_string().split('-').next().unwrap()
    ))
    .bind(format!(
        "user{}@test.com",
        user_id.to_string().split('-').next().unwrap()
    ))
    .bind(password_hash)
    .bind(format!("+{:012}", user_id.as_u128() % 1000000000000))
    .bind(role_str)
    .bind(status_str)
    .bind("Test User")
    .bind(org_id)
    .bind(badge_id)
    .execute(db)
    .await
    .unwrap();

    user_id
}

fn issue_admin_token(jwt_private_key_pem: &str, admin_id: Uuid) -> String {
    let jwt_svc = JwtService::new(jwt_private_key_pem).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let shift_end = now + 3600;

    jwt_svc
        .issue_access_token_with_shift(
            admin_id,
            Uuid::nil(), // admin has no device
            UserRole::Admin,
            now.try_into().unwrap(),
            shift_end.try_into().unwrap(),
        )
        .unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// User CRUD Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_provision_user_creates_new_user() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let provision_body = json!({
        "username": "newagent",
        "email": "newagent@test.com",
        "fullName": "New Agent User",
        "phoneNumber": "+1234567890",
        "role": "agent",
        "organizationId": org_id,
        "badgeId": "BADGE-12345"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/users")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(provision_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "provision_user should return 201"
    );

    // Verify response body
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["user"]["username"], "newagent");
    assert_eq!(body["user"]["email"], "newagent@test.com");
    // superadmin endpoint always creates org_admin regardless of requested role
    assert_eq!(body["user"]["role"], "org_admin");
    assert_eq!(body["user"]["status"], "ACTIVE");
    // temp password must be present
    assert!(
        body["tempPassword"].is_string(),
        "tempPassword should be returned"
    );
}

#[tokio::test]
async fn test_provision_user_requires_admin_role() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let agent_token = {
        let jwt_svc = JwtService::new(&jwt_private_key_pem).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let shift_end = now + 3600;

        jwt_svc
            .issue_access_token_with_shift(
                agent_id,
                Uuid::nil(),
                UserRole::Agent,
                now.try_into().unwrap(),
                shift_end.try_into().unwrap(),
            )
            .unwrap()
    };

    let provision_body = json!({
        "username": "newuser",
        "email": "newuser@test.com",
        "fullName": "New User",
        "phoneNumber": "+1234567890",
        "role": "agent",
        "organizationId": org_id,
        "badgeId": "BADGE-12345"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/users")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", agent_token))
                .body(Body::from(provision_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "non-admin should get 403 when trying to provision user"
    );
}

#[tokio::test]
async fn test_list_users_returns_all_users() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let _agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let _manager_id = create_test_user(&db, org_id, UserRole::Manager, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/users")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "list_users should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let users = body.as_array().unwrap();
    assert!(
        users.len() >= 2,
        "Should return at least 2 users (agent + manager, excluding the requesting admin)"
    );
}

#[tokio::test]
async fn test_get_user_returns_specific_user() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/admin/users/{}", agent_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "get_user should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["id"], agent_id.to_string());
    assert_eq!(body["role"], "agent");
}

#[tokio::test]
async fn test_get_user_returns_404_for_nonexistent_user() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);
    let nonexistent_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/admin/users/{}", nonexistent_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "get_user should return 404 for nonexistent user"
    );
}

#[tokio::test]
async fn test_update_user_updates_fields() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let update_body = json!({
        "fullName": "Updated Agent Name",
        "phoneNumber": "+9876543210"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}", agent_id))
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "update_user should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["name"], "Updated Agent Name");
    assert_eq!(body["phoneNumber"], "+9876543210");
}

#[tokio::test]
async fn test_delete_user_removes_user() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/admin/users/{}", agent_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "delete_user should return 204"
    );

    // Verify user is deleted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(agent_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 0, "User should be deleted from database");
}

// ─────────────────────────────────────────────────────────────────────────────
// Admin Reactivation Tests (via update_user with status change)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_update_user_can_reactivate_suspended_admin() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let suspended_admin_id =
        create_test_user(&db, org_id, UserRole::Admin, UserStatus::Suspended).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    // First verify the admin is suspended
    let status_before: String = sqlx::query_scalar("SELECT status::TEXT FROM users WHERE id = $1")
        .bind(suspended_admin_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        status_before, "SUSPENDED",
        "Admin should be suspended initially"
    );

    // Reactivate via update_user
    let update_body = json!({
        "status": "ACTIVE"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}", suspended_admin_id))
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "update_user for reactivation should return 200"
    );

    // Verify admin is now active
    let status_after: String = sqlx::query_scalar("SELECT status::TEXT FROM users WHERE id = $1")
        .bind(suspended_admin_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(status_after, "ACTIVE", "Admin should be reactivated");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["status"], "ACTIVE");
}

#[tokio::test]
async fn test_update_user_can_suspend_active_admin() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let target_admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    // Suspend via update_user
    let update_body = json!({
        "status": "SUSPENDED"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/admin/users/{}", target_admin_id))
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "update_user for suspension should return 200"
    );

    // Verify admin is now suspended
    let status_after: String = sqlx::query_scalar("SELECT status::TEXT FROM users WHERE id = $1")
        .bind(target_admin_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(status_after, "SUSPENDED", "Admin should be suspended");
}

// ─────────────────────────────────────────────────────────────────────────────
// User Profile Tests (from handlers/users.rs)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_current_user_profile() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;

    // Create device for agent
    let device_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO devices (id, user_id, public_key, status, created_at) VALUES ($1, $2, 'test-public-key', 'ACTIVE', NOW())"#,
    )
    .bind(device_id)
    .bind(agent_id)
    .execute(&db)
    .await
    .unwrap();

    let agent_token = {
        let jwt_svc = JwtService::new(&jwt_private_key_pem).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let shift_end = now + 3600;

        jwt_svc
            .issue_access_token_with_shift(
                agent_id,
                device_id,
                UserRole::Agent,
                now.try_into().unwrap(),
                shift_end.try_into().unwrap(),
            )
            .unwrap()
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", agent_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "get_current_user should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["id"], agent_id.to_string());
    assert_eq!(body["role"], "agent");
}

// ─────────────────────────────────────────────────────────────────────────────
// RBAC Tests for User Management
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_manager_cannot_provision_admin_user() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let manager_id = create_test_user(&db, org_id, UserRole::Manager, UserStatus::Active).await;

    let manager_token = {
        let jwt_svc = JwtService::new(&jwt_private_key_pem).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let shift_end = now + 3600;

        jwt_svc
            .issue_access_token_with_shift(
                manager_id,
                Uuid::nil(),
                UserRole::Manager,
                now.try_into().unwrap(),
                shift_end.try_into().unwrap(),
            )
            .unwrap()
    };

    let provision_body = json!({
        "username": "newadmin",
        "email": "newadmin@test.com",
        "fullName": "New Admin User",
        "phoneNumber": "+1234567890",
        "role": "admin",
        "organizationId": org_id,
        "badgeId": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/users")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", manager_token))
                .body(Body::from(provision_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Manager should get 403 when trying to create an admin
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "manager should not be able to provision admin users"
    );
}

#[tokio::test]
async fn test_agent_cannot_access_user_management_endpoints() {
    let (db, _cache, app, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;

    let agent_token = {
        let jwt_svc = JwtService::new(&jwt_private_key_pem).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let shift_end = now + 3600;

        jwt_svc
            .issue_access_token_with_shift(
                agent_id,
                Uuid::nil(),
                UserRole::Agent,
                now.try_into().unwrap(),
                shift_end.try_into().unwrap(),
            )
            .unwrap()
    };

    // Try to list users
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/users")
                .header("Authorization", format!("Bearer {}", agent_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "agent should not be able to list users"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Device Activation Tests (activate function)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_activate_success() {
    let (db, cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let org_id = create_test_organization(&db).await;

    // Create agent user with PENDING_ACTIVATION status
    let agent_id = {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, phone_number, 
                role, status, full_name, created_at, organization_id, badge_id
            )
            VALUES ($1, $2, $3, $4, 'agent'::user_role, 'PENDING_ACTIVATION'::user_status, $5, NOW(), $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(format!("agent_{}", user_id.to_string().split('-').next().unwrap()))
        .bind(format!("agent{}@test.com", user_id.to_string().split('-').next().unwrap()))
        .bind("+1234567890")
        .bind("Test Agent")
        .bind(org_id)
        .bind("AGENT-TEST-001")
        .execute(&db)
        .await
        .unwrap();
        user_id
    };

    let device_id = Uuid::new_v4();
    let public_key_base64 = generate_test_public_key_base64();
    let test_otp = "123456";

    // Store OTP in Moka cache using the app's cache
    store_test_otp(&cache, agent_id, test_otp)
        .await
        .expect("Failed to store test OTP");

    let activate_body = json!({
        "badgeId": "AGENT-TEST-001",
        "activationCode": test_otp,
        "deviceId": device_id,
        "publicKeyBase64": public_key_base64
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "activate should return 200 for valid request"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response structure
    assert!(body["accessToken"].is_string(), "Should have accessToken");
    assert!(body["refreshToken"].is_string(), "Should have refreshToken");
    assert!(body["user"].is_object(), "Should have user object");
    assert_eq!(body["user"]["role"], "agent", "User role should be agent");

    // Verify user status changed to ACTIVE
    let status: String = sqlx::query_scalar("SELECT status::TEXT FROM users WHERE id = $1")
        .bind(agent_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(status, "ACTIVE", "User should be ACTIVE after activation");

    // Verify device was created
    let device_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM devices WHERE id = $1 AND user_id = $2)")
            .bind(device_id)
            .bind(agent_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert!(device_exists, "Device should be created");
}

#[tokio::test]
async fn test_activate_missing_badge_id() {
    let (_db, _cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let device_id = Uuid::new_v4();

    let activate_body = json!({
        "badgeId": "",
        "activationCode": "123456",
        "deviceId": device_id,
        "publicKeyBase64": "dGVzdGtleQ=="
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "activate should return 400 for missing badgeId"
    );
}

#[tokio::test]
async fn test_activate_missing_activation_code() {
    let (_db, _cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let device_id = Uuid::new_v4();

    let activate_body = json!({
        "badgeId": "AGENT-TEST-001",
        "activationCode": "",
        "deviceId": device_id,
        "publicKeyBase64": "dGVzdGtleQ=="
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "activate should return 400 for missing activationCode"
    );
}

#[tokio::test]
async fn test_activate_invalid_base64_public_key() {
    let (_db, _cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let device_id = Uuid::new_v4();

    let activate_body = json!({
        "badgeId": "AGENT-TEST-001",
        "activationCode": "123456",
        "deviceId": device_id,
        "publicKeyBase64": "not-valid-base64!!!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "activate should return 400 for invalid base64 public key"
    );
}

#[tokio::test]
async fn test_activate_user_not_found() {
    let (_db, _cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let device_id = Uuid::new_v4();

    let activate_body = json!({
        "badgeId": "NON-EXISTENT-BADGE",
        "activationCode": "123456",
        "deviceId": device_id,
        "publicKeyBase64": "dGVzdGtleQ=="
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "activate should return 404 for non-existent user"
    );
}

#[tokio::test]
async fn test_activate_non_agent_user() {
    let (db, _cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;

    // Create admin user with PENDING_ACTIVATION status
    let _admin_with_badge_id = {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, phone_number, 
                role, status, full_name, created_at, organization_id, badge_id
            )
            VALUES ($1, $2, $3, $4, 'admin'::user_role, 'PENDING_ACTIVATION'::user_status, $5, NOW(), $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(format!("admin_{}", user_id.to_string().split('-').next().unwrap()))
        .bind(format!("admin{}@test.com", user_id.to_string().split('-').next().unwrap()))
        .bind("+1234567891")
        .bind("Test Admin")
        .bind(org_id)
        .bind("ADMIN-TEST-001")
        .execute(&db)
        .await
        .unwrap();
        user_id
    };

    let device_id = Uuid::new_v4();

    let activate_body = json!({
        "badgeId": "ADMIN-TEST-001",
        "activationCode": "123456",
        "deviceId": device_id,
        "publicKeyBase64": "dGVzdGtleQ=="
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "activate should return 400 for non-agent users"
    );
}

#[tokio::test]
async fn test_activate_user_not_pending_activation() {
    let (db, _cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;

    // Create agent user with ACTIVE status (not PENDING_ACTIVATION)
    let _agent_id = {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, phone_number, 
                role, status, full_name, created_at, organization_id, badge_id
            )
            VALUES ($1, $2, $3, $4, 'agent'::user_role, 'ACTIVE'::user_status, $5, NOW(), $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(format!(
            "agent_{}",
            user_id.to_string().split('-').next().unwrap()
        ))
        .bind(format!(
            "agent{}@test.com",
            user_id.to_string().split('-').next().unwrap()
        ))
        .bind("+1234567892")
        .bind("Test Agent Active")
        .bind(org_id)
        .bind("AGENT-ACTIVE-001")
        .execute(&db)
        .await
        .unwrap();
        user_id
    };

    let device_id = Uuid::new_v4();

    let activate_body = json!({
        "badgeId": "AGENT-ACTIVE-001",
        "activationCode": "123456",
        "deviceId": device_id,
        "publicKeyBase64": "dGVzdGtleQ=="
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "activate should return 400 for user not in PENDING_ACTIVATION status"
    );
}

#[tokio::test]
async fn test_activate_invalid_otp() {
    let (db, cache, app, _jwt_private_key_pem, _jwt_public_key_pem, _postgres) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;

    // Create agent user with PENDING_ACTIVATION status
    let agent_id = {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, phone_number, 
                role, status, full_name, created_at, organization_id, badge_id
            )
            VALUES ($1, $2, $3, $4, 'agent'::user_role, 'PENDING_ACTIVATION'::user_status, $5, NOW(), $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(format!("agent_{}", user_id.to_string().split('-').next().unwrap()))
        .bind(format!("agent{}@test.com", user_id.to_string().split('-').next().unwrap()))
        .bind("+1234567893")
        .bind("Test Agent")
        .bind(org_id)
        .bind("AGENT-OTP-001")
        .execute(&db)
        .await
        .unwrap();
        user_id
    };

    let device_id = Uuid::new_v4();

    // Store correct OTP in cache using the app's cache
    store_test_otp(&cache, agent_id, "123456")
        .await
        .expect("Failed to store test OTP");

    // Try to activate with WRONG OTP
    let activate_body = json!({
        "badgeId": "AGENT-OTP-001",
        "activationCode": "000000",
        "deviceId": device_id,
        "publicKeyBase64": "dGVzdGtleQ=="
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(activate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Note: The handler wraps OTP validation errors in BadRequest (400), not Unauthorized (401)
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "activate should return 400 for invalid OTP"
    );
}
