//! Integration tests for stats handler endpoints.
//!
//! Tests the HTTP handlers in `crate::handlers::stats`:
//! - GET /admin/stats (requires admin role)
//! - GET /stats/activity
//! - GET /stats/top-agents
//! - GET /stats/activity-feed
//! - GET /stats/recent-alerts

use crate::app_state::AppState;
use crate::routes;
use crate::services::sms_provider::MockSmsProvider;
use crate::telemetry::TelemetryHandle;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use rand::rngs::OsRng;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper: builds a full AppState + Axum app backed by real Postgres + Moka cache.
async fn setup_test_app() -> (
    axum::Router,
    sqlx::PgPool,
    std::sync::Arc<crate::app_cache::AppCache>,
    String,
    String,
    testcontainers::ContainerAsync<Postgres>,
) {
    let pg = Postgres::default().with_host_auth().start().await.unwrap();
    let pg_port = pg.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres@127.0.0.1:{}/postgres", pg_port);

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    let config = crate::config::Config {
        database_url: db_url.clone(),
        server_host: "127.0.0.1".into(),
        server_port: 0,
        log_level: crate::config::LogLevel::Info,
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: crate::config::Environment::Local,
        sms_credentials: crate::config::SmsProviderCredentials::Mock,
        email_credentials: crate::config::EmailProviderCredentials::Mock,
        otp_via_email: false,
        activation_code_pepper: "test_pepper_for_activation_code_hashing_must_be_32_chars_long"
            .to_string(),
        admin_bootstrap_email: Some("admin@example.com".to_string()),
        admin_bootstrap_password: Some("password".to_string()),
        admin_bootstrap_phone: Some("1234567890".to_string()),
        admin_bootstrap_username: Some("admin".to_string()),
        vehicle_api_credentials: crate::config::mock_vehicle_api_credentials(),
    };

    let cache = std::sync::Arc::new(crate::app_cache::AppCache::new());
    let state = AppState::new(
        db.clone(),
        cache.clone(),
        Arc::new(MockSmsProvider),
        Arc::new(crate::services::email_provider::MockEmailProvider),
        &config,
        Arc::new(TelemetryHandle::noop()),
    )
    .await.expect("failed to initialize test app state");

    let app = routes::assembly(state.clone());

    (
        app,
        db,
        state.app_cache.clone(),
        jwt_private_key_pem,
        jwt_public_key_pem,
        pg,
    )
}

fn generate_test_rsa_keypair_pem() -> (String, String) {
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

/// Helper: seed an organization and admin user with access token.
async fn seed_admin_user(db: &sqlx::PgPool, jwt_private_key_pem: &str) -> (Uuid, String) {
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type, start_work_time, end_work_time) VALUES ($1, $2, $3, $4, $5)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .bind(360i32)
        .bind(1080i32)
        .execute(db)
        .await
        .unwrap();

    // Create admin user
    let admin_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status, password_hash)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status, $9)
        "#,
    )
    .bind(admin_id)
    .bind(org_id)
    .bind("admin001")
    .bind("admin001@example.com")
    .bind("admin")
    .bind("ADMIN-001")
    .bind("Admin User")
    .bind("+237600000000")
    .bind("dummy_hash")  // Admin users need password_hash
    .execute(db)
    .await
    .unwrap();

    // Issue a JWT access token for the admin
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let shift_end = now_secs + 8 * 3600;

    let jwt_svc = crate::services::jwt_service::JwtService::new(jwt_private_key_pem).unwrap();
    let access_token = jwt_svc
        .issue_access_token_with_shift(
            admin_id,
            Uuid::nil(), // Admin has no device
            crate::dto::users::UserRole::Admin,
            now_secs.try_into().unwrap(),
            shift_end.try_into().unwrap(),
        )
        .unwrap();

    (admin_id, access_token)
}

/// Helper: seed an organization and an org_admin user.
async fn seed_org_admin(
    db: &sqlx::PgPool,
    jwt_private_key_pem: &str,
    org_name: &str,
) -> (Uuid, Uuid, String) {
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type, start_work_time, end_work_time) VALUES ($1, $2, $3, $4, $5)"#)
        .bind(org_id)
        .bind(org_name)
        .bind("police")
        .bind(360i32)
        .bind(1080i32)
        .execute(db)
        .await
        .unwrap();

    let admin_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, role, full_name, email, status, password_hash, phone_number)
        VALUES ($1, $2, $3, 'org_admin'::user_role, $4, $5, 'ACTIVE'::user_status, $6, $7)
        "#,
    )
    .bind(admin_id)
    .bind(org_id)
    .bind(format!("{}_admin", org_name.to_lowercase().replace(' ', "_")))
    .bind(format!("{} Admin", org_name))
    .bind(format!("admin@{}", org_name.to_lowercase().replace(' ', "")))
    .bind("dummy_hash")
    .bind(format!("+237{}", Uuid::new_v4().to_string().chars().filter(|c| c.is_ascii_digit()).collect::<String>().chars().take(9).collect::<String>()))
    .execute(db)
    .await
    .unwrap();

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let shift_end = now_secs + 8 * 3600;

    let jwt_svc = crate::services::jwt_service::JwtService::new(jwt_private_key_pem).unwrap();
    let access_token = jwt_svc
        .issue_access_token_with_shift(
            admin_id,
            Uuid::nil(),
            crate::dto::users::UserRole::OrgAdmin,
            now_secs.try_into().unwrap(),
            shift_end.try_into().unwrap(),
        )
        .unwrap();

    (org_id, admin_id, access_token)
}

/// Helper: seed an organization and agent with device and refresh token.
async fn seed_test_data(
    db: &sqlx::PgPool,
    jwt_private_key_pem: &str,
) -> (Uuid, Uuid, Uuid, String) {
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type, start_work_time, end_work_time) VALUES ($1, $2, $3, $4, $5)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .bind(360i32)
        .bind(1080i32)
        .execute(db)
        .await
        .unwrap();

    // Create agent user
    let agent_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4::user_role, $5, $6, $7, 'ACTIVE'::user_status)
        "#,
    )
    .bind(agent_id)
    .bind(org_id)
    .bind("agent001")
    .bind("agent")
    .bind("AGENT-001")
    .bind("Agent 001")
    .bind("+237600000001")
    .execute(db)
    .await
    .unwrap();

    // Register a device for the agent
    let device_id = Uuid::new_v4();
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let shift_end = now_secs + 8 * 3600;

    let public_key_base64 = base64::engine::general_purpose::STANDARD.encode(b"fake-public-key");
    sqlx::query(
        r#"
        INSERT INTO devices (id, user_id, public_key, status, metadata)
        VALUES ($1, $2, $3, 'ACTIVE'::device_status,
                jsonb_build_object('shift_start', $4, 'shift_end', $5))
        "#,
    )
    .bind(device_id)
    .bind(agent_id)
    .bind(&public_key_base64)
    .bind(now_secs)
    .bind(shift_end)
    .execute(db)
    .await
    .unwrap();

    // Issue a JWT access token
    let jwt_svc = crate::services::jwt_service::JwtService::new(jwt_private_key_pem).unwrap();
    let access_token = jwt_svc
        .issue_access_token_with_shift(
            agent_id,
            device_id,
            crate::dto::users::UserRole::Agent,
            now_secs.try_into().unwrap(),
            shift_end.try_into().unwrap(),
        )
        .unwrap();

    (org_id, agent_id, device_id, access_token)
}

/// Helper: seed control records for testing stats.
async fn seed_control_records(db: &sqlx::PgPool, agent_id: Uuid, org_id: Uuid) {
    // Seed multiple control records with different statuses
    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address)
        VALUES ($1, $2, $3, $4, NOW(), $5, 'Test Address')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_id)
    .bind(org_id)
    .bind("AB-123-CD")
    .bind("valid")
    .execute(db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address)
        VALUES ($1, $2, $3, $4, NOW(), $5, 'Test Address')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_id)
    .bind(org_id)
    .bind("EF-456-GH")
    .bind("warning")
    .execute(db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address)
        VALUES ($1, $2, $3, $4, NOW(), $5, 'Test Address')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_id)
    .bind(org_id)
    .bind("IJ-789-KL")
    .bind("critical")
    .execute(db)
    .await
    .unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /admin/stats Tests (requires admin role)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_dashboard_stats_success() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    // Seed admin user (required for /admin/stats)
    let (_admin_id, admin_access_token) = seed_admin_user(&db, &jwt_private_key_pem).await;

    // Also seed some control records
    let (org_id, agent_id, _device_id, _agent_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/stats")
                .header("Authorization", format!("Bearer {}", admin_access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /admin/stats should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response structure matches DashboardStats (uses camelCase)
    assert!(
        body["todayControls"].is_number(),
        "Response should contain todayControls"
    );
    assert!(
        body["activeAlerts"].is_number(),
        "Response should contain activeAlerts"
    );
    assert!(
        body["totalVehicles"].is_number(),
        "Response should contain totalVehicles"
    );
    assert!(
        body["onlineAgents"].is_number(),
        "Response should contain onlineAgents"
    );
    assert!(
        body["pendingSubmissions"].is_number(),
        "Response should contain pendingSubmissions"
    );
    assert!(
        body["organizationsCount"].is_number(),
        "Response should contain organizationsCount"
    );
    assert!(
        body["activity24h"].is_array(),
        "Response should contain activity24h"
    );
    assert!(
        body["liveAgents"].is_array(),
        "Response should contain liveAgents"
    );
}

#[tokio::test]
async fn test_get_dashboard_stats_empty_database() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    // Seed admin user (required for /admin/stats)
    let (_admin_id, admin_access_token) = seed_admin_user(&db, &jwt_private_key_pem).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/stats")
                .header("Authorization", format!("Bearer {}", admin_access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /admin/stats should return 200 even with no data"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify all fields are present (using camelCase)
    assert!(
        body["todayControls"].is_number(),
        "todayControls should be a number"
    );
    assert!(
        body["activeAlerts"].is_number(),
        "activeAlerts should be a number"
    );
}

#[tokio::test]
async fn test_get_dashboard_stats_unauthorized() {
    let (app, _db, _cache, _jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "GET /admin/stats without token should return 401"
    );
}

#[tokio::test]
async fn test_get_dashboard_stats_forbidden_for_agent() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    // Seed agent user (not admin)
    let (_org_id, _agent_id, _device_id, agent_access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/admin/stats")
                .header("Authorization", format!("Bearer {}", agent_access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "GET /admin/stats with agent token should return 403"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /stats/activity Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_control_activity_default_range() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/activity")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/activity should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Range is serialized as enum with serde(rename = "24h")
    assert_eq!(
        body["range"].as_str().unwrap(),
        "24h",
        "Default range should be 24h"
    );
    assert!(
        body["series"].is_array(),
        "Response should contain series array"
    );

    let series = body["series"].as_array().unwrap();
    assert_eq!(
        series.len(),
        24,
        "24h range should return 24 hourly buckets"
    );

    // Verify series structure
    if !series.is_empty() {
        let point = &series[0];
        assert!(point["label"].is_string(), "Point should have label");
        assert!(point["count"].is_number(), "Point should have count");
    }
}

#[tokio::test]
async fn test_get_control_activity_7d_range() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/activity?range=7d")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/activity?range=7d should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["range"].as_str().unwrap(), "7d", "Range should be 7d");
    let series = body["series"].as_array().unwrap();
    assert_eq!(series.len(), 7, "7d range should return 7 daily buckets");
}

#[tokio::test]
async fn test_get_control_activity_30d_range() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/activity?range=30d")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/activity?range=30d should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(
        body["range"].as_str().unwrap(),
        "30d",
        "Range should be 30d"
    );
    let series = body["series"].as_array().unwrap();
    assert_eq!(series.len(), 30, "30d range should return 30 daily buckets");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /stats/top-agents Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_top_agents_default() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/top-agents")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/top-agents should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Range is serialized as enum with serde(rename = "24h")
    assert_eq!(
        body["range"].as_str().unwrap(),
        "24h",
        "Default range should be 24h"
    );
    assert!(
        body["agents"].is_array(),
        "Response should contain agents array"
    );
    let agents = body["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1, "Should return 1 agent");

    // Verify agent structure (using camelCase due to #[serde(rename_all = "camelCase")])
    let agent = &agents[0];
    assert!(agent["agentId"].is_string(), "Agent should have agentId");
    assert!(
        agent["agentName"].is_string(),
        "Agent should have agentName"
    );
    assert!(
        agent["organizationName"].is_string(),
        "Agent should have organizationName"
    );
    assert!(
        agent["controlsCount"].is_number(),
        "Agent should have controlsCount"
    );
    assert!(agent["isOnline"].is_boolean(), "Agent should have isOnline");
    assert_eq!(
        agent["controlsCount"].as_i64().unwrap(),
        3,
        "Agent should have 3 controls"
    );
}

#[tokio::test]
async fn test_get_top_agents_with_limit() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/top-agents?limit=5")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/top-agents?limit=5 should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    let agents = body["agents"].as_array().unwrap();
    assert_eq!(
        agents.len(),
        1,
        "Should return 1 agent (limited by actual data)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /stats/activity-feed Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_activity_feed_default() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/activity-feed")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/activity-feed should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(
        body["items"].is_array(),
        "Response should contain items array"
    );
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "Should return 3 feed items");

    // Verify item structure (using camelCase due to #[serde(rename_all = "camelCase")])
    let item = &items[0];
    assert!(item["id"].is_string(), "Item should have id");
    assert!(
        item["plateNumber"].is_string(),
        "Item should have plateNumber"
    );
    assert!(
        item["overallStatus"].is_string(),
        "Item should have overallStatus"
    );
    assert!(item["createdAt"].is_string(), "Item should have createdAt");
    assert!(item["agentName"].is_string(), "Item should have agentName");
}

#[tokio::test]
async fn test_get_activity_feed_with_limit() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/activity-feed?limit=2")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/activity-feed?limit=2 should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "Should return only 2 items due to limit");
}

#[tokio::test]
async fn test_get_activity_feed_empty() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    // Seed only basic user data (no control records)
    let (_org_id, _agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/activity-feed")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/activity-feed should return 200 even with no data"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    let items = body["items"].as_array().unwrap();
    assert!(items.is_empty(), "Should return empty array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /stats/recent-alerts Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_recent_alerts_default() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/recent-alerts")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /api/v1/stats/recent-alerts should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(
        body["items"].is_array(),
        "Response should contain items array"
    );
    let items = body["items"].as_array().unwrap();
    // Should only include warning and critical, not valid
    assert_eq!(
        items.len(),
        2,
        "Should return 2 alerts (warning + critical)"
    );

    // Verify item structure (using camelCase due to #[serde(rename_all = "camelCase")])
    let item = &items[0];
    assert!(item["id"].is_string(), "Item should have id");
    assert!(
        item["plateNumber"].is_string(),
        "Item should have plateNumber"
    );
    assert!(
        item["overallStatus"].is_string(),
        "Item should have overallStatus"
    );
    assert!(
        item["overallStatus"].as_str().unwrap() != "valid",
        "Alert should not be valid status"
    );
    assert!(item["createdAt"].is_string(), "Item should have createdAt");
    assert!(item["agentName"].is_string(), "Item should have agentName");
}

#[tokio::test]
async fn test_get_recent_alerts_with_limit() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;
    seed_control_records(&db, agent_id, org_id).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/recent-alerts?limit=1")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/recent-alerts?limit=1 should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "Should return only 1 alert due to limit");
}

#[tokio::test]
async fn test_get_recent_alerts_no_alerts() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    let (org_id, agent_id, _device_id, access_token) =
        seed_test_data(&db, &jwt_private_key_pem).await;

    // Seed only valid records (no alerts)
    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address)
        VALUES ($1, $2, $3, $4, NOW(), 'valid', 'Test Address')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_id)
    .bind(org_id)
    .bind("AB-123-CD")
    .execute(&db)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/stats/recent-alerts")
                .header("Authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats/recent-alerts should return 200 even with no alerts"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    let items = body["items"].as_array().unwrap();
    assert!(
        items.is_empty(),
        "Should return empty array when no alerts exist"
    );
}

#[tokio::test]
async fn test_get_org_dashboard_stats_data_isolation() {
    let (app, db, _cache, jwt_private_key_pem, _jwt_public_key_pem, _pg) = setup_test_app().await;

    // 1. Seed Org A and its Admin
    let (_org_a_id, _admin_a_id, access_token_a) =
        seed_org_admin(&db, &jwt_private_key_pem, "Org A").await;

    // 2. Seed Org B and some data (1 control, 1 alert)
    let (org_b_id, _admin_b_id, _access_token_b) =
        seed_org_admin(&db, &jwt_private_key_pem, "Org B").await;

    // Create an agent for Org B
    let agent_b_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, 'agent'::user_role, $4, $5, $6, 'ACTIVE'::user_status)
        "#,
    )
    .bind(agent_b_id)
    .bind(org_b_id)
    .bind("agent_b")
    .bind("B-001")
    .bind("Agent B")
    .bind("+237600000002")
    .execute(&db)
    .await
    .unwrap();

    // Add a control record for Org B
    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address)
        VALUES ($1, $2, $3, $4, NOW(), 'critical', 'Org B Location')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_b_id)
    .bind(org_b_id)
    .bind("ORG-B-123")
    .execute(&db)
    .await
    .unwrap();

    // 3. Call Org A's dashboard stats - should see 0 controls/alerts
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/org-admin/stats")
                .header("Authorization", format!("Bearer {}", access_token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(
        body["todayControls"], 0,
        "Org A should see 0 controls from Org B"
    );
    assert_eq!(
        body["activeAlerts"], 0,
        "Org A should see 0 alerts from Org B"
    );
    assert_eq!(body["organizationName"], "Org A");

    // 4. Call Org B's dashboard stats (we need to manually construct a request for B if we wanted,
    // but the isolation for A is the key test here).
}
