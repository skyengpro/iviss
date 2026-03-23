//! Integration tests for auth_queries module.
//!
//! Tests the repository functions in `crate::queries::auth_queries`:
//! - mark_device_inactive
//! - mark_device_active
//! - suspend_device_and_revoke_tokens
//! - blacklist_jti
//! - has_valid_refresh_token

use crate::queries::auth_queries;
use base64::Engine;
use rand::rngs::OsRng;
use sha2::Digest;
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use time::Duration;
use time::OffsetDateTime;
use time::PrimitiveDateTime;
use uuid::Uuid;

/// Helper: builds a full AppState + Axum app backed by real Postgres + Redis.
async fn setup_test_infrastructure() -> (
    sqlx::PgPool,
    deadpool_redis::Pool,
    String,
    String,
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

    (
        db,
        redis_pool,
        jwt_private_key_pem,
        jwt_public_key_pem,
        pg,
        redis_container,
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

/// Helper: seed a device for a user.
async fn seed_device(
    db: &sqlx::PgPool,
    user_id: Uuid,
    status: &str,
) -> Uuid {
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
        VALUES ($1, $2, $3, $4::device_status,
                jsonb_build_object('shift_start', $5, 'shift_end', $6))
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .bind(&public_key_base64)
    .bind(status)
    .bind(now_secs)
    .bind(shift_end)
    .execute(db)
    .await
    .expect("Failed to seed device");

    device_id
}

/// Helper: create a valid refresh token for a device.
async fn seed_refresh_token(
    db: &sqlx::PgPool,
    user_id: Uuid,
    device_id: Uuid,
    revoked: bool,
) {
    let token = format!("refresh-token-{}", Uuid::new_v4());
    let token_hash = format!("{:x}", sha2::Sha256::digest(token.as_bytes()));
    let refresh_expires = OffsetDateTime::now_utc() + Duration::days(30);
    let refresh_expires_primitive = PrimitiveDateTime::new(refresh_expires.date(), refresh_expires.time());

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .bind(device_id)
    .bind(refresh_expires_primitive)
    .bind(revoked)
    .execute(db)
    .await
    .expect("Failed to seed refresh token");
}

// ─────────────────────────────────────────────────────────────────────────────
// mark_device_inactive Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mark_device_inactive_success() {
    let (db, _redis_pool, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent001")
    .bind("agent@example.com")
    .bind("agent")
    .bind("AGENT-001")
    .bind("Agent 001")
    .bind("+237600000001")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;

    // Verify device is active
    let device_before: (String,) = sqlx::query_as(
        r#"SELECT status::text FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device");
    assert_eq!(device_before.0, "ACTIVE");

    // Call the function under test
    let result = auth_queries::mark_device_inactive(&db, device_id).await;
    assert!(result.is_ok(), "mark_device_inactive should succeed");

    // Verify device is now inactive
    let device_after: (String, Option<time::PrimitiveDateTime>) = sqlx::query_as(
        r#"SELECT status::text, revoked_at FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device after update");
    assert_eq!(device_after.0, "INACTIVE");
    assert!(device_after.1.is_some(), "revoked_at should be set");
}

#[tokio::test]
async fn test_mark_device_inactive_nonexistent_device() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let nonexistent_id = Uuid::new_v4();

    // Should not error - UPDATE just affects 0 rows
    let result = auth_queries::mark_device_inactive(&db, nonexistent_id).await;
    assert!(result.is_ok(), "mark_device_inactive should succeed even for nonexistent device");
}

#[tokio::test]
async fn test_mark_device_inactive_already_inactive() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent002")
    .bind("agent002@example.com")
    .bind("agent")
    .bind("AGENT-002")
    .bind("Agent 002")
    .bind("+237600000002")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "INACTIVE").await;

    // Should not error - UPDATE just affects 0 rows (WHERE status = 'ACTIVE' fails)
    let result = auth_queries::mark_device_inactive(&db, device_id).await;
    assert!(result.is_ok(), "mark_device_inactive should succeed even if already inactive");

    // Verify device is still inactive
    let device_after: (String,) = sqlx::query_as(
        r#"SELECT status::text FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device");
    assert_eq!(device_after.0, "INACTIVE");
}

// ─────────────────────────────────────────────────────────────────────────────
// mark_device_active Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mark_device_active_success() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent003")
    .bind("agent003@example.com")
    .bind("agent")
    .bind("AGENT-003")
    .bind("Agent 003")
    .bind("+237600000003")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "INACTIVE").await;
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let shift_start = now_secs;
    let shift_end = now_secs + 8 * 3600;

    // Call the function under test
    let result = auth_queries::mark_device_active(&db, device_id, shift_start, shift_end).await;
    assert!(result.is_ok(), "mark_device_active should succeed");

    // Verify device is now active with correct metadata
    let device: (String, sqlx::types::Json<serde_json::Value>) = sqlx::query_as(
        r#"SELECT status::text, metadata FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device");
    assert_eq!(device.0, "ACTIVE");
    let metadata = device.1 .0;
    assert_eq!(metadata["shift_start"], shift_start);
    assert_eq!(metadata["shift_end"], shift_end);
}

#[tokio::test]
async fn test_mark_device_active_updates_existing_device() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent004")
    .bind("agent004@example.com")
    .bind("agent")
    .bind("AGENT-004")
    .bind("Agent 004")
    .bind("+237600000004")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "SUSPENDED").await;
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let shift_start = now_secs;
    let shift_end = now_secs + 10 * 3600; // Different shift duration

    // Call the function under test
    let result = auth_queries::mark_device_active(&db, device_id, shift_start, shift_end).await;
    assert!(result.is_ok(), "mark_device_active should succeed");

    // Verify device status changed from SUSPENDED to ACTIVE
    let device: (String,) = sqlx::query_as(
        r#"SELECT status::text FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device");
    assert_eq!(device.0, "ACTIVE");
}

// ─────────────────────────────────────────────────────────────────────────────
// suspend_device_and_revoke_tokens Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_suspend_device_and_revoke_tokens_success() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent005")
    .bind("agent005@example.com")
    .bind("agent")
    .bind("AGENT-005")
    .bind("Agent 005")
    .bind("+237600000005")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;

    // Create multiple refresh tokens (some valid, some expired)
    seed_refresh_token(&db, user_id, device_id, false).await;
    seed_refresh_token(&db, user_id, device_id, false).await;
    seed_refresh_token(&db, user_id, device_id, false).await;

    // Verify tokens are not revoked
    let tokens_before: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM refresh_tokens WHERE device_id = $1 AND revoked = FALSE"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to count tokens");
    assert_eq!(tokens_before.0, 3);

    // Call the function under test
    let result = auth_queries::suspend_device_and_revoke_tokens(&db, device_id).await;
    assert!(result.is_ok(), "suspend_device_and_revoke_tokens should succeed");

    // Verify device is suspended
    let device: (String, Option<time::PrimitiveDateTime>) = sqlx::query_as(
        r#"SELECT status::text, revoked_at FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device");
    assert_eq!(device.0, "SUSPENDED");
    assert!(device.1.is_some(), "revoked_at should be set");

    // Verify tokens are now revoked
    let tokens_after: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM refresh_tokens WHERE device_id = $1 AND revoked = FALSE"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to count tokens");
    assert_eq!(tokens_after.0, 0);

    // Verify revoked tokens have timestamps
    let revoked_tokens: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM refresh_tokens WHERE device_id = $1 AND revoked = TRUE AND revoked_at IS NOT NULL"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to count revoked tokens");
    assert_eq!(revoked_tokens.0, 3);
}

#[tokio::test]
async fn test_suspend_device_and_revoke_tokens_no_tokens() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent006")
    .bind("agent006@example.com")
    .bind("agent")
    .bind("AGENT-006")
    .bind("Agent 006")
    .bind("+237600000006")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;

    // Call the function under test (no tokens exist for this device)
    let result = auth_queries::suspend_device_and_revoke_tokens(&db, device_id).await;
    assert!(result.is_ok(), "suspend_device_and_revoke_tokens should succeed with no tokens");

    // Verify device is still suspended
    let device: (String,) = sqlx::query_as(
        r#"SELECT status::text FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to fetch device");
    assert_eq!(device.0, "SUSPENDED");
}

#[tokio::test]
async fn test_suspend_device_and_revoke_tokens_only_revokes_valid_tokens() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent007")
    .bind("agent007@example.com")
    .bind("agent")
    .bind("AGENT-007")
    .bind("Agent 007")
    .bind("+237600000007")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;

    // Create one valid token and one already revoked token
    seed_refresh_token(&db, user_id, device_id, false).await;
    seed_refresh_token(&db, user_id, device_id, true).await;

    // Call the function under test
    let result = auth_queries::suspend_device_and_revoke_tokens(&db, device_id).await;
    assert!(result.is_ok(), "suspend_device_and_revoke_tokens should succeed");

    // Verify only the valid token was revoked
    let valid_tokens: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM refresh_tokens WHERE device_id = $1 AND revoked = FALSE"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to count valid tokens");
    assert_eq!(valid_tokens.0, 0);

    let all_revoked: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM refresh_tokens WHERE device_id = $1 AND revoked = TRUE"#,
    )
    .bind(device_id)
    .fetch_one(&db)
    .await
    .expect("Failed to count all revoked tokens");
    assert_eq!(all_revoked.0, 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// blacklist_jti Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_blacklist_jti_success() {
    let (_, redis_pool, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let jti = Uuid::new_v4().to_string();
    let ttl_secs = 3600u64;

    // Call the function under test
    let result = auth_queries::blacklist_jti(&redis_pool, &jti, ttl_secs).await;
    assert!(result.is_ok(), "blacklist_jti should succeed");

    // Verify the key exists in Redis
    let mut conn = redis_pool.get().await.expect("Failed to get Redis connection");
    let exists: bool = redis::cmd("EXISTS")
        .arg(format!("blacklist:jti:{}", jti))
        .query_async(&mut *conn)
        .await
        .expect("Failed to query Redis");
    assert!(exists, "Blacklisted JTI should exist in Redis");
}

#[tokio::test]
async fn test_blacklist_jti_expiration() {
    let (_, redis_pool, _, _, _pg, _redis) = setup_test_infrastructure().await;
    let _ = redis_pool; // Used for verification after TTL expires

    let jti = Uuid::new_v4().to_string();
    let ttl_secs = 1u64; // Very short TTL for testing

    // Call the function under test
    let result = auth_queries::blacklist_jti(&redis_pool, &jti, ttl_secs).await;
    assert!(result.is_ok(), "blacklist_jti should succeed");

    // Verify the key exists immediately
    let mut conn = redis_pool.get().await.expect("Failed to get Redis connection");
    let value_before: String = redis::cmd("GET")
        .arg(format!("blacklist:jti:{}", jti))
        .query_async(&mut *conn)
        .await
        .expect("Failed to query Redis");
    assert_eq!(value_before, "1", "Blacklisted JTI should have value '1'");

    // Wait for TTL to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Verify the key no longer exists
    let exists_after: bool = redis::cmd("EXISTS")
        .arg(format!("blacklist:jti:{}", jti))
        .query_async(&mut *conn)
        .await
        .expect("Failed to query Redis");
    assert!(!exists_after, "Blacklisted JTI should have expired");
}

#[tokio::test]
async fn test_blacklist_jti_multiple_calls() {
    let (_, redis_pool, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let jti1 = Uuid::new_v4().to_string();
    let jti2 = Uuid::new_v4().to_string();
    let ttl_secs = 3600u64;

    // Blacklist multiple JTIs
    let result1 = auth_queries::blacklist_jti(&redis_pool, &jti1, ttl_secs).await;
    let result2 = auth_queries::blacklist_jti(&redis_pool, &jti2, ttl_secs).await;
    assert!(result1.is_ok(), "blacklist_jti should succeed for jti1");
    assert!(result2.is_ok(), "blacklist_jti should succeed for jti2");

    // Verify both exist
    let mut conn = redis_pool.get().await.expect("Failed to get Redis connection");
    let exists1: bool = redis::cmd("EXISTS")
        .arg(format!("blacklist:jti:{}", jti1))
        .query_async(&mut *conn)
        .await
        .expect("Failed to query Redis");
    let exists2: bool = redis::cmd("EXISTS")
        .arg(format!("blacklist:jti:{}", jti2))
        .query_async(&mut *conn)
        .await
        .expect("Failed to query Redis");
    assert!(exists1, "jti1 should be blacklisted");
    assert!(exists2, "jti2 should be blacklisted");
}

// ─────────────────────────────────────────────────────────────────────────────
// has_valid_refresh_token Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_has_valid_refresh_token_true() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent008")
    .bind("agent008@example.com")
    .bind("agent")
    .bind("AGENT-008")
    .bind("Agent 008")
    .bind("+237600000008")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;
    seed_refresh_token(&db, user_id, device_id, false).await;

    // Call the function under test
    let result = auth_queries::has_valid_refresh_token(&db, device_id).await;
    assert!(result.is_ok(), "has_valid_refresh_token should succeed");
    assert!(result.unwrap(), "Should return true when valid token exists");
}

#[tokio::test]
async fn test_has_valid_refresh_token_false_no_token() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent009")
    .bind("agent009@example.com")
    .bind("agent")
    .bind("AGENT-009")
    .bind("Agent 009")
    .bind("+237600000009")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;
    // No refresh token created

    // Call the function under test
    let result = auth_queries::has_valid_refresh_token(&db, device_id).await;
    assert!(result.is_ok(), "has_valid_refresh_token should succeed");
    assert!(!result.unwrap(), "Should return false when no token exists");
}

#[tokio::test]
async fn test_has_valid_refresh_token_false_revoked() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind("police")
        .execute(&db)
        .await
        .expect("Failed to seed org");

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind("agent010")
    .bind("agent010@example.com")
    .bind("agent")
    .bind("AGENT-010")
    .bind("Agent 010")
    .bind("+237600000010")
    .execute(&db)
    .await
    .expect("Failed to seed user");

    let device_id = seed_device(&db, user_id, "ACTIVE").await;
    seed_refresh_token(&db, user_id, device_id, true).await; // revoked = true

    // Call the function under test
    let result = auth_queries::has_valid_refresh_token(&db, device_id).await;
    assert!(result.is_ok(), "has_valid_refresh_token should succeed");
    assert!(!result.unwrap(), "Should return false when token is revoked");
}

#[tokio::test]
async fn test_has_valid_refresh_token_false_nonexistent_device() {
    let (db, _, _, _, _pg, _redis) = setup_test_infrastructure().await;

    let nonexistent_device_id = Uuid::new_v4();

    // Call the function under test
    let result = auth_queries::has_valid_refresh_token(&db, nonexistent_device_id).await;
    assert!(result.is_ok(), "has_valid_refresh_token should succeed");
    assert!(!result.unwrap(), "Should return false for nonexistent device");
}
