use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    config::{Config, Environment, LogLevel},
    db::redis::RedisPool,
    dto::users::{UserRole, UserStatus},
    routes,
    services::jwt_service::JwtService,
};

use testcontainers_modules::{
    postgres::Postgres, redis::Redis, testcontainers::runners::AsyncRunner,
};

// ─────────────────────────────────────────────────────────────────────────────
// Test Infrastructure Setup
// ─────────────────────────────────────────────────────────────────────────────

// Use existing test fixtures for RSA keys
const TEST_PRIVATE_KEY: &str = include_str!("fixtures/test_private_key.pem");
const TEST_PUBLIC_KEY: &str = include_str!("fixtures/test_public_key.pem");

fn generate_test_rsa_keypair_pem() -> (String, String) {
    (TEST_PRIVATE_KEY.to_string(), TEST_PUBLIC_KEY.to_string())
}

async fn setup_test_app() -> (
    PgPool,
    RedisPool,
    Router,
    String, // jwt_private_key_pem
    String, // jwt_public_key_pem
    testcontainers::ContainerAsync<Postgres>,
    testcontainers::ContainerAsync<Redis>,
) {
    // Start PostgreSQL container
    let postgres = Postgres::default().start().await.unwrap();
    let pg_host = postgres.get_host().await.unwrap();
    let pg_port = postgres.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres:postgres@{pg_host}:{pg_port}/postgres");

    // Start Redis container
    let redis = Redis::default().start().await.unwrap();
    let redis_host = redis.get_host().await.unwrap();
    let redis_port = redis.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://{redis_host}:{redis_port}");

    // Create database pool
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    // Create Redis pool
    let redis_pool = crate::db::redis::initialize_redis_pool(&redis_url)
        .await
        .unwrap();

    // Generate test keys
    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    // Create test config
    let config = Config {
        database_url: db_url,
        redis_url,
        server_host: "0.0.0.0".to_string(),
        server_port: 8080,
        log_level: LogLevel::Info,
        jwt_secret: "test_secret".to_string(),
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: Environment::Local,
        activation_code_pepper: "test_pepper".to_string(),
        twilio_account_sid: "mock".to_string(),
        twilio_auth_token: "mock".to_string(),
        twilio_from_number: "mock".to_string(),
        admin_bootstrap_email: Some("admin@test.com".to_string()),
        admin_bootstrap_password: Some("password123".to_string()),
        admin_bootstrap_phone: Some("+1234567890".to_string()),
        admin_bootstrap_username: Some("admin".to_string()),
        shift_start_hour: 8,
        shift_end_hour: 18,
    };

    // Create app state with mock SMS provider
    let sms_provider: Arc<dyn crate::services::sms_provider::SmsProvider> =
        Arc::new(crate::services::sms_provider::MockSmsProvider);
    let state = AppState::new(db.clone(), redis_pool.clone(), sms_provider, &config);

    // Build router
    let app = routes::assembly(state);

    (
        db,
        redis_pool,
        app,
        jwt_private_key_pem,
        jwt_public_key_pem,
        postgres,
        redis,
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri("/admin/users")
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
    assert_eq!(body["username"], "newagent");
    assert_eq!(body["email"], "newagent@test.com");
    assert_eq!(body["role"], "agent");
    assert_eq!(body["status"], "PENDING_ACTIVATION");
}

#[tokio::test]
async fn test_provision_user_requires_admin_role() {
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri("/admin/users")
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let _agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let _manager_id = create_test_user(&db, org_id, UserRole::Manager, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/users")
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
        users.len() >= 3,
        "Should return at least 3 users (admin + agent + manager)"
    );
}

#[tokio::test]
async fn test_get_user_returns_specific_user() {
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/users/{}", agent_id))
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);
    let nonexistent_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/users/{}", nonexistent_id))
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri(format!("/admin/users/{}", agent_id))
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

    let org_id = create_test_organization(&db).await;
    let admin_id = create_test_user(&db, org_id, UserRole::Admin, UserStatus::Active).await;
    let agent_id = create_test_user(&db, org_id, UserRole::Agent, UserStatus::Active).await;
    let admin_token = issue_admin_token(&jwt_private_key_pem, admin_id);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/{}", agent_id))
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri(format!("/admin/users/{}", suspended_admin_id))
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri(format!("/admin/users/{}", target_admin_id))
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri("/users/me")
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri("/admin/users")
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
    let (db, _redis, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis_container) =
        setup_test_app().await;

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
                .uri("/admin/users")
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
