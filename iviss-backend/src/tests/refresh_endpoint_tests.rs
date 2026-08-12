use crate::app_state::AppState;
use crate::routes;
use crate::services::notifications::email_provider::MockEmailProvider;
use crate::services::notifications::sms_provider::MockSmsProvider;
use crate::telemetry::TelemetryHandle;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use p256::ecdsa::{signature::Signer, SigningKey};
use rand::rngs::OsRng;
use serde_json::json;
use sha2::Digest;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;
use uuid::Uuid;

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

/// Helper: create a Base64-encoded JWK public key string from a p256 signing key
fn ec_public_key_to_b64_jwk(signing_key: &SigningKey) -> String {
    let public_key = p256::PublicKey::from(signing_key.verifying_key());
    let jwk_str = public_key.to_jwk_string();
    base64::engine::general_purpose::STANDARD.encode(jwk_str.as_bytes())
}

/// Helper: produce a compact JWS (ES256) of a nonce using the given signing key
fn sign_nonce_jws(nonce: &str, signing_key: &SigningKey) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    // Header: {"alg":"ES256"}
    let header_b64 = URL_SAFE_NO_PAD.encode(br#"{"alg":"ES256"}"#);

    // Payload: base64url(nonce)
    let payload_b64 = URL_SAFE_NO_PAD.encode(nonce.as_bytes());

    // Signing input
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    // Sign
    let signature: p256::ecdsa::Signature = signing_key.sign(signing_input.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

    format!("{}.{}.{}", header_b64, payload_b64, sig_b64)
}

/// Setup helper that creates all infrastructure and returns (app, db, user_id, device_id,
/// refresh_token, ec_signing_key)
async fn setup_test_infrastructure() -> (
    axum::Router,
    sqlx::PgPool,
    Uuid,
    Uuid,
    String,
    SigningKey,
    // Keep containers alive
    testcontainers::ContainerAsync<Postgres>,
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

    // Create organization
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type, start_work_time, end_work_time) VALUES ($1, $2, $3, $4, $5)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .bind(360i32)
        .bind(1080i32)
        .execute(&db)
        .await
        .unwrap();

    // Create user
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4::user_role, $5, $6, $7, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent001")
    .bind("agent")
    .bind("AGENT-001")
    .bind("Agent 001")
    .bind("+237600000001")
    .execute(&db)
    .await
    .unwrap();

    // Generate EC key pair for device
    let ec_signing_key = SigningKey::random(&mut OsRng);
    let public_key_b64 = ec_public_key_to_b64_jwk(&ec_signing_key);

    let device_id = Uuid::new_v4();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let shift_start = now;
    let shift_end = now + 3600;

    sqlx::query(
        r#"
        INSERT INTO devices (id, user_id, public_key, status, metadata)
        VALUES ($1, $2, $3, 'ACTIVE'::device_status, jsonb_build_object('shift_start', $4, 'shift_end', $5))
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .bind(&public_key_b64)
    .bind(shift_start as i64)
    .bind(shift_end as i64)
    .execute(&db)
    .await
    .unwrap();

    // Create refresh token
    let refresh_token = {
        let mut raw = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut raw);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
    };
    let refresh_token_hash = format!("{:x}", sha2::Sha256::digest(refresh_token.as_bytes()));
    let refresh_expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&refresh_token_hash)
    .bind(user_id)
    .bind(device_id)
    .bind(refresh_expires_at)
    .execute(&db)
    .await
    .unwrap();

    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    let config = crate::config::Config {
        database_url: db_url,
        server_host: "0.0.0.0".to_string(),
        server_port: 0,
        log_level: crate::config::LogLevel::Info,
        jwt_private_key_pem: jwt_private_key_pem.clone(),
        jwt_public_key_pem: jwt_public_key_pem.clone(),
        environment: crate::config::Environment::Local,
        cors_allowed_origins: vec!["http://localhost:8080".to_string()],
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
        enable_vehicle_api: true,
        s3_cache: crate::config::S3CacheConfig::default(),
    };

    let state = AppState::new(
        db.clone(),
        Arc::new(crate::app_cache::AppCache::new()),
        Arc::new(MockSmsProvider),
        Arc::new(MockEmailProvider),
        &config,
        Arc::new(TelemetryHandle::noop()),
        None,
    )
    .expect("failed to initialize test app state");

    let app = routes::assembly(Arc::new(state));

    (
        app,
        db,
        user_id,
        device_id,
        refresh_token,
        ec_signing_key,
        pg,
    )
}

#[tokio::test]
async fn test_refresh_flow_success() {
    let (app, _db, _user_id, device_id, refresh_token, ec_signing_key, _pg) =
        setup_test_infrastructure().await;

    // Step 1: Request a nonce via /auth/refresh
    let refresh_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let nonce = body["nonce"].as_str().unwrap().to_string();
    assert!(!nonce.is_empty());

    // Step 2: Sign the nonce and verify via /auth/refresh/verify
    let signed_nonce = sign_nonce_jws(&nonce, &ec_signing_key);

    let verify_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
        "signedNonce": signed_nonce,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh/verify")
                .header("content-type", "application/json")
                .body(Body::from(verify_body.to_string()))
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
    assert!(!access_token.is_empty());
}

#[tokio::test]
async fn test_refresh_with_invalid_token() {
    let (app, _db, _user_id, device_id, _refresh_token, _ec_signing_key, _pg) =
        setup_test_infrastructure().await;

    let refresh_body = json!({
        "refreshToken": "completely-invalid-token",
        "deviceId": device_id,
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Regression: after a failed verify_refresh (bad signature, revoked
/// token), the nonce must remain available so a legitimate retry with the
/// correct signature still works. Previously the nonce was invalidated up
/// front, so any failure burned a valid challenge that the real device
/// could otherwise have completed.
#[tokio::test]
async fn test_verify_refresh_keeps_nonce_available_on_bad_signature() {
    let (app, _db, _user_id, device_id, refresh_token, ec_signing_key, _pg) =
        setup_test_infrastructure().await;

    let refresh_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
    });

    let response = app
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
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let nonce = body["nonce"].as_str().unwrap().to_string();

    // First verify with the WRONG key — must fail without consuming the nonce.
    let wrong_key = SigningKey::random(&mut OsRng);
    let bad_signed = sign_nonce_jws(&nonce, &wrong_key);
    let bad_verify_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
        "signedNonce": bad_signed,
    });
    let bad_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh/verify")
                .header("content-type", "application/json")
                .body(Body::from(bad_verify_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_resp.status(), StatusCode::UNAUTHORIZED);

    // Now retry with the CORRECT signature — since the nonce wasn't burned
    // by the bad attempt, this must succeed.
    let good_signed = sign_nonce_jws(&nonce, &ec_signing_key);
    let good_verify_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
        "signedNonce": good_signed,
    });
    let good_resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh/verify")
                .header("content-type", "application/json")
                .body(Body::from(good_verify_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(good_resp.status(), StatusCode::OK);
}

/// Regression: replaying the exact same signed nonce twice must fail the
/// second time. The atomic `remove()` at the end of verify_refresh returns
/// `None` on the second call, and the endpoint responds with a retriable
/// NONCE_RETRY error rather than issuing a second access token.
#[tokio::test]
async fn test_verify_refresh_rejects_replayed_signed_nonce() {
    let (app, _db, _user_id, device_id, refresh_token, ec_signing_key, _pg) =
        setup_test_infrastructure().await;

    // Step 1: get a nonce.
    let refresh_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
    });
    let response = app
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
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let nonce = body["nonce"].as_str().unwrap().to_string();

    let signed_nonce = sign_nonce_jws(&nonce, &ec_signing_key);
    let verify_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
        "signedNonce": signed_nonce,
    });

    // First verify: succeeds and consumes the nonce.
    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh/verify")
                .header("content-type", "application/json")
                .body(Body::from(verify_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    // Second verify with the exact same signed nonce: must be rejected.
    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh/verify")
                .header("content-type", "application/json")
                .body(Body::from(verify_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::UNAUTHORIZED);

    let second_body_bytes = axum::body::to_bytes(second.into_body(), usize::MAX)
        .await
        .unwrap();
    let second_body: serde_json::Value = serde_json::from_slice(&second_body_bytes).unwrap();
    // Must carry the retriable NONCE_RETRY code so the frontend re-challenges
    // instead of logging the user out.
    assert_eq!(second_body["code"].as_str(), Some("NONCE_RETRY"));
}

#[tokio::test]
async fn test_refresh_with_invalid_signature() {
    let (app, _db, _user_id, device_id, refresh_token, _ec_signing_key, _pg) =
        setup_test_infrastructure().await;

    // Step 1: Get a valid nonce
    let refresh_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let nonce = body["nonce"].as_str().unwrap().to_string();

    // Step 2: Sign the nonce with a DIFFERENT key (not the registered one)
    let wrong_key = SigningKey::random(&mut OsRng);
    let signed_nonce = sign_nonce_jws(&nonce, &wrong_key);

    let verify_body = json!({
        "refreshToken": refresh_token,
        "deviceId": device_id,
        "signedNonce": signed_nonce,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh/verify")
                .header("content-type", "application/json")
                .body(Body::from(verify_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
