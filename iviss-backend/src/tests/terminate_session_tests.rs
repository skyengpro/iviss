use crate::app_state::AppState;
use crate::routes;
use crate::services::sms_provider::MockSmsProvider;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use rand::rngs::OsRng;
use serde_json::json;
use sha2::Digest;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_PEPPER: &str = "test_pepper_for_activation_code_hashing_must_be_32_chars_long";

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

/// Helper: builds a full AppState + Axum app backed by real Postgres + Redis.
async fn setup_test_app() -> (
    sqlx::PgPool,
    deadpool_redis::Pool,
    axum::Router,
    String, // jwt_private_key_pem
    String, // jwt_public_key_pem
    testcontainers::ContainerAsync<Postgres>,
    testcontainers::ContainerAsync<Redis>,
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

    let redis_container = Redis::default().start().await.unwrap();
    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://127.0.0.1:{}", redis_port);
    let redis_cfg = deadpool_redis::Config::from_url(redis_url.clone());
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap();

    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    let config = crate::config::Config {
        database_url: db_url,
        redis_url,
        server_host: "127.0.0.1".into(),
        server_port: 0,
        log_level: crate::config::LogLevel::Info,
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: crate::config::Environment::Local,
        twilio_account_sid: "mock".into(),
        twilio_auth_token: "mock".into(),
        twilio_from_number: "mock".into(),
        activation_code_pepper: TEST_PEPPER.to_string(),
        shift_start_hour: 8,
        shift_end_hour: 20,
        admin_bootstrap_email: Some("admin@example.com".to_string()),
        admin_bootstrap_password: Some("admin123".to_string()),
        admin_bootstrap_phone: Some("+1234567890".to_string()),
        admin_bootstrap_username: Some("admin".to_string()),
    };

    let state = AppState::new(
        db.clone(),
        redis_pool.clone(),
        Arc::new(MockSmsProvider),
        &config,
    );

    let app = routes::assembly(state);

    (
        db,
        redis_pool,
        app,
        jwt_private_key_pem,
        jwt_public_key_pem,
        pg,
        redis_container,
    )
}

/// Helper: seed an organization, admin user, and agent user (with device + refresh token).
/// Returns (org_id, admin_user_id, agent_user_id, device_id, admin_access_token, agent_access_token).
async fn seed_users_with_active_session(
    db: &sqlx::PgPool,
    jwt_private_key_pem: &str,
) -> (Uuid, Uuid, Uuid, Uuid, String, String) {
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(db)
        .await
        .unwrap();

    // Create admin user
    let admin_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
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

    // Create a refresh token for the agent
    let refresh_token = "test-refresh-token-value";
    let refresh_token_hash = format!("{:x}", sha2::Sha256::digest(refresh_token.as_bytes()));
    let refresh_expires = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&refresh_token_hash)
    .bind(agent_id)
    .bind(device_id)
    .bind(refresh_expires)
    .execute(db)
    .await
    .unwrap();

    // Issue a JWT access token for the admin
    let jwt_svc = crate::services::jwt_service::JwtService::new(jwt_private_key_pem).unwrap();
    let admin_access_token = jwt_svc
        .issue_access_token_with_shift(
            admin_id,
            Uuid::nil(), // admin has no device
            crate::dto::users::UserRole::Admin,
            now_secs.try_into().unwrap(),
            shift_end.try_into().unwrap(),
        )
        .unwrap();

    // Issue a JWT access token for the agent
    let agent_access_token = jwt_svc
        .issue_access_token_with_shift(
            agent_id,
            device_id,
            crate::dto::users::UserRole::Agent,
            now_secs.try_into().unwrap(),
            shift_end.try_into().unwrap(),
        )
        .unwrap();

    (
        org_id,
        admin_id,
        agent_id,
        device_id,
        admin_access_token,
        agent_access_token,
    )
}

#[tokio::test]
async fn terminate_session_revokes_tokens_and_deactivates_devices() {
    let (db, _redis_pool, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis) =
        setup_test_app().await;

    let (_org_id, _admin_id, agent_id, device_id, admin_access_token, _agent_access_token) =
        seed_users_with_active_session(&db, &jwt_private_key_pem).await;

    // -- 1. Call POST /admin/terminate-session ──
    let terminate_body = json!({ "userId": agent_id });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/terminate-session")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_access_token))
                .body(Body::from(terminate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "terminate-session should return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains(&agent_id.to_string()),
        "response should mention the terminated user id"
    );

    // ── 2. Verify DB state ──

    // User status should remain ACTIVE (termination does not suspend the account)
    let user_status: String = sqlx::query_scalar("SELECT status::TEXT FROM users WHERE id = $1")
        .bind(agent_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        user_status, "ACTIVE",
        "user status should remain ACTIVE after session termination"
    );

    // All refresh tokens should be revoked
    let active_tokens: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1 AND revoked = FALSE",
    )
    .bind(agent_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(active_tokens, 0, "all refresh tokens should be revoked");

    // Device should be INACTIVE
    let device_status: String =
        sqlx::query_scalar("SELECT status::TEXT FROM devices WHERE id = $1")
            .bind(device_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(device_status, "INACTIVE", "device should be INACTIVE");

    // ── 3. Verify that a subsequent authenticated request returns 401 ──
    let protected_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/users/me")
                .header("Authorization", format!("Bearer {}", _agent_access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        protected_response.status(),
        StatusCode::UNAUTHORIZED,
        "subsequent request with old JWT should return 401"
    );
}

#[tokio::test]
async fn terminate_session_rejects_non_agent_user() {
    let (db, _redis_pool, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis) =
        setup_test_app().await;

    let (_org_id, admin_id, _agent_id, _device_id, _admin_access_token, _agent_access_token) =
        seed_users_with_active_session(&db, &jwt_private_key_pem).await;

    // Try to terminate the admin's session — should fail with 400
    let terminate_body = json!({ "userId": admin_id });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/terminate-session")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", _admin_access_token))
                .body(Body::from(terminate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "terminate-session should reject non-agent users"
    );
}

#[tokio::test]
async fn terminate_session_returns_404_for_unknown_user() {
    let (db, _redis_pool, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis) =
        setup_test_app().await;

    // Seed some data so the app is initialized properly
    let (_org_id, _admin_id, _agent_id, _device_id, admin_access_token, _agent_access_token) =
        seed_users_with_active_session(&db, &jwt_private_key_pem).await;

    let unknown_id = Uuid::new_v4();
    let terminate_body = json!({ "userId": unknown_id });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/terminate-session")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_access_token))
                .body(Body::from(terminate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "terminate-session should return 404 for unknown user"
    );
}

#[tokio::test]
async fn terminate_session_blocks_otp_on_same_day() {
    let (db, _redis_pool, app, jwt_private_key_pem, _jwt_public_key_pem, _pg, _redis) =
        setup_test_app().await;

    let (_org_id, _admin_id, agent_id, device_id, admin_access_token, _agent_access_token) =
        seed_users_with_active_session(&db, &jwt_private_key_pem).await;

    // -- 1. Terminate session ──
    let terminate_body = json!({ "userId": agent_id });
    let _terminate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/terminate-session")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_access_token))
                .body(Body::from(terminate_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // -- 2. Try to request a daily login OTP ──
    let otp_body = json!({
        "badgeId": "AGENT-001",
        "deviceId": device_id.to_string(),
    });

    let otp_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/request-daily-login")
                .header("content-type", "application/json")
                .body(Body::from(otp_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let otp_status = otp_response.status();
    let body_bytes = axum::body::to_bytes(otp_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    if otp_status != StatusCode::FORBIDDEN {
        println!("Error Response Body: {:?}", body);
    }

    assert_eq!(
        otp_status,
        StatusCode::FORBIDDEN,
        "OTP request should be blocked on the same day as termination"
    );

    let message = body["message"].as_str().unwrap_or("");
    let message_lower = message.to_lowercase();
    assert!(
        message_lower.contains("wait until") || message_lower.contains("next allowed window"),
        "error message should mention the waiting period, got: {}",
        message
    );
}
