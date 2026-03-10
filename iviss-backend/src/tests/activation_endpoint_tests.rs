use crate::app_state::AppState;
use crate::routes;
use crate::services::activation_service::ActivationService;
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

#[tokio::test]
async fn activation_flow_activates_user_and_issues_tokens() {
    let pg = Postgres::default().start().await.unwrap();
    let pg_port = pg.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        pg_port
    );

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
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

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .unwrap();

    let user_id = Uuid::new_v4();
    let badge_id = "AGENT-001";
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4::user_role, $5, $6, $7, 'PENDING_ACTIVATION'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent001")
    .bind("agent")
    .bind(badge_id)
    .bind("Agent 001")
    .bind("+237600000001")
    .execute(&db)
    .await
    .unwrap();

    let activation_svc = ActivationService::new(
        redis_pool.clone(),
        Arc::new(MockSmsProvider),
        TEST_PEPPER.to_string(),
    );
    let otp = activation_svc.generate_and_store(&user_id).await.unwrap();

    let device_id = Uuid::new_v4();
    let public_key_base64 = base64::engine::general_purpose::STANDARD.encode(b"fake-public-key");

    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    let config = crate::config::Config {
        database_url: db_url.clone(),
        redis_url: redis_url.clone(),
        server_host: "127.0.0.1".into(),
        server_port: 0,
        log_level: crate::config::LogLevel::Info,
        jwt_secret: "dummy_testing_value_long_enough_to_pass_validation".into(),
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: crate::config::Environment::Local,
        twilio_account_sid: "mock".into(),
        twilio_auth_token: "mock".into(),
        twilio_from_number: "mock".into(),
        activation_code_pepper: TEST_PEPPER.to_string(),
        shift_start_hour: 8,
        shift_end_hour: 18,
    };

    let state = AppState::new(
        db.clone(),
        redis_pool.clone(),
        Arc::new(MockSmsProvider),
        config,
    );

    let app = routes::assembly(state);

    let req_body = json!({
        "badgeId": badge_id,
        "activationCode": otp,
        "deviceId": device_id,
        "publicKeyBase64": public_key_base64,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/activate")
                .header("content-type", "application/json")
                .body(Body::from(req_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    let access_token = body["accessToken"].as_str().unwrap().to_string();
    let refresh_token = body["refreshToken"].as_str().unwrap().to_string();

    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());

    let status: String = sqlx::query_scalar("SELECT status::TEXT FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(status, "ACTIVE");

    let device_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM devices WHERE id = $1 AND user_id = $2")
            .bind(device_id)
            .bind(user_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(device_count, 1);

    let stored_hash: String = sqlx::query_scalar(
        "SELECT token_hash FROM refresh_tokens WHERE user_id = $1 AND device_id = $2 AND revoked = FALSE",
    )
    .bind(user_id)
    .bind(device_id)
    .fetch_one(&db)
    .await
    .unwrap();

    let expected_hash = format!("{:x}", sha2::Sha256::digest(refresh_token.as_bytes()));
    assert_eq!(stored_hash, expected_hash);
    assert_ne!(stored_hash, refresh_token);
}
