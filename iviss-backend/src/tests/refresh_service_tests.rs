use crate::db::RedisPool;
use crate::dto::auth::AccessTokenClaims;
use crate::errors::AppError;
use crate::services::refresh_service::{RefreshService, ACCESS_TOKEN_TTL_SECS};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use deadpool_redis::{Config as RedisConfig, Runtime};
use ed25519_dalek::{Signer, SigningKey};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use sqlx::PgPool;
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use testcontainers_modules::redis::Redis;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "be06-refresh-jwt-secret-must-be-at-least-32-chars";
const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://iviss_user:iviss_password@127.0.0.1:5435/iviss_db";
const DEFAULT_TEST_REDIS_URL: &str = "redis://127.0.0.1:6380";

struct TestContext {
    _pg_container: Option<ContainerAsync<GenericImage>>,
    _redis_container: Option<ContainerAsync<Redis>>,
    db_pool: PgPool,
    redis_pool: RedisPool,
}

struct SeedData {
    user_id: Uuid,
    device_id: Uuid,
    refresh_token: String,
    signing_key: SigningKey,
}

async fn setup_context() -> TestContext {
    let database_url = std::env::var("BE06_TEST_DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
    let redis_url =
        std::env::var("BE06_TEST_REDIS_URL").unwrap_or_else(|_| DEFAULT_TEST_REDIS_URL.to_string());

    // Prefer already-running local services; this avoids requiring Docker socket
    // access for every local test run.
    if let Ok(context) = try_setup_external_context(&database_url, &redis_url).await {
        return context;
    }

    let pg_image = GenericImage::new("postgres", "15-alpine")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "iviss")
        .with_mapped_port(0, 5432.tcp());
    let pg_container = pg_image.start().await.unwrap();
    let pg_port = pg_container.get_host_port_ipv4(5432).await.unwrap();

    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{pg_port}/iviss");
    let db_pool = connect_with_retry(&database_url)
        .await
        .unwrap_or_else(|err| {
            panic!("Failed to connect to container postgres after retries: {err}")
        });
    sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();

    let redis_container = Redis::default().start().await.unwrap();
    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://127.0.0.1:{redis_port}");
    let redis_pool = RedisConfig::from_url(redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    TestContext {
        _pg_container: Some(pg_container),
        _redis_container: Some(redis_container),
        db_pool,
        redis_pool,
    }
}

async fn try_setup_external_context(
    database_url: &str,
    redis_url: &str,
) -> Result<TestContext, String> {
    let db_pool = connect_with_retry(database_url)
        .await
        .map_err(|err| format!("database connect failed: {err}"))?;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .map_err(|err| format!("migration failed: {err}"))?;

    let redis_pool = RedisConfig::from_url(redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|err| format!("redis pool creation failed: {err}"))?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| format!("redis connection failed: {err}"))?;
    let _: String = redis::cmd("PING")
        .query_async(&mut redis_conn)
        .await
        .map_err(|err| format!("redis ping failed: {err}"))?;

    Ok(TestContext {
        _pg_container: None,
        _redis_container: None,
        db_pool,
        redis_pool,
    })
}

async fn connect_with_retry(database_url: &str) -> Result<PgPool, sqlx::Error> {
    for attempt in 1..=10 {
        match PgPool::connect(database_url).await {
            Ok(pool) => return Ok(pool),
            Err(err) => {
                if attempt == 10 {
                    return Err(err);
                }
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
    unreachable!("retry loop should always return");
}

async fn seed_active_refresh_token(pool: &PgPool) -> SeedData {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let refresh_token = format!("refresh-{}", Uuid::new_v4());
    let refresh_token_hash = RefreshService::hash_refresh_token(&refresh_token);
    let username = format!("be06-agent-{}", &user_id.simple().to_string()[..8]);
    let badge_id = format!("AG-{}", &user_id.simple().to_string()[..6].to_uppercase());
    let phone_suffix: u32 = rand::random::<u32>() % 10_000_000;
    let phone_number = format!("+2376{phone_suffix:07}");

    let secret_key: [u8; 32] = rand::random();
    let signing_key = SigningKey::from_bytes(&secret_key);
    let public_key = STANDARD.encode(signing_key.verifying_key().as_bytes());

    sqlx::query(
        r#"
        INSERT INTO organizations (id, name, type, region, deleted_at)
        VALUES ($1, 'BE06 Test Org', 'police', 'Centre', NULL)
        "#,
    )
    .bind(organization_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO users (
            id, organization_id, username, email, password_hash, role, badge_id, full_name, phone_number, status, deleted_at
        )
        VALUES (
            $1, $2, $3, NULL, NULL, 'agent'::user_role, $4, 'BE06 Agent', $5, 'ACTIVE'::user_status, NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(organization_id)
    .bind(username)
    .bind(badge_id)
    .bind(phone_number)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO devices (id, user_id, public_key, metadata, status)
        VALUES ($1, $2, $3, '{}'::jsonb, 'ACTIVE'::device_status)
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .bind(public_key)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at, revoked)
        VALUES ($1, $2, $3, NOW() + INTERVAL '30 days', FALSE)
        "#,
    )
    .bind(refresh_token_hash)
    .bind(user_id)
    .bind(device_id)
    .execute(pool)
    .await
    .unwrap();

    SeedData {
        user_id,
        device_id,
        refresh_token,
        signing_key,
    }
}

#[tokio::test]
async fn refresh_flow_issues_access_token_after_valid_signature() {
    let ctx = setup_context().await;
    let seed = seed_active_refresh_token(&ctx.db_pool).await;
    let refresh_service = RefreshService::new(
        ctx.db_pool.clone(),
        ctx.redis_pool.clone(),
        TEST_JWT_SECRET.to_string(),
    );

    let challenge = refresh_service
        .create_nonce_challenge(&seed.refresh_token, seed.device_id)
        .await
        .unwrap();
    let signature = seed.signing_key.sign(challenge.nonce.as_bytes());
    let signature_b64 = STANDARD.encode(signature.to_bytes());

    let access_token = refresh_service
        .verify_and_issue_access_token(
            &seed.refresh_token,
            seed.device_id,
            challenge.challenge_id,
            &signature_b64,
        )
        .await
        .unwrap();

    let decoded = decode::<AccessTokenClaims>(
        &access_token,
        &DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .unwrap();

    assert_eq!(decoded.claims.sub, seed.user_id.to_string());
    assert_eq!(decoded.claims.device_id, seed.device_id);
    assert_eq!(access_token.split('.').count(), 3);
    assert!(
        decoded.claims.exp - decoded.claims.iat <= ACCESS_TOKEN_TTL_SECS as i64,
        "access token lifetime should stay within configured TTL"
    );
}

#[tokio::test]
async fn refresh_flow_rejects_nonce_replay() {
    let ctx = setup_context().await;
    let seed = seed_active_refresh_token(&ctx.db_pool).await;
    let refresh_service = RefreshService::new(
        ctx.db_pool.clone(),
        ctx.redis_pool.clone(),
        TEST_JWT_SECRET.to_string(),
    );

    let challenge = refresh_service
        .create_nonce_challenge(&seed.refresh_token, seed.device_id)
        .await
        .unwrap();
    let signature = seed.signing_key.sign(challenge.nonce.as_bytes());
    let signature_b64 = STANDARD.encode(signature.to_bytes());

    refresh_service
        .verify_and_issue_access_token(
            &seed.refresh_token,
            seed.device_id,
            challenge.challenge_id,
            &signature_b64,
        )
        .await
        .unwrap();

    let replay_attempt = refresh_service
        .verify_and_issue_access_token(
            &seed.refresh_token,
            seed.device_id,
            challenge.challenge_id,
            &signature_b64,
        )
        .await;

    assert!(matches!(
        replay_attempt,
        Err(AppError::Unauthorized(message))
            if message.contains("already used") || message.contains("replay")
    ));
}

#[tokio::test]
async fn refresh_flow_revokes_token_when_signature_is_invalid() {
    let ctx = setup_context().await;
    let seed = seed_active_refresh_token(&ctx.db_pool).await;
    let refresh_service = RefreshService::new(
        ctx.db_pool.clone(),
        ctx.redis_pool.clone(),
        TEST_JWT_SECRET.to_string(),
    );

    let challenge = refresh_service
        .create_nonce_challenge(&seed.refresh_token, seed.device_id)
        .await
        .unwrap();

    let wrong_secret_key: [u8; 32] = rand::random();
    let wrong_signing_key = SigningKey::from_bytes(&wrong_secret_key);
    let wrong_signature = wrong_signing_key.sign(challenge.nonce.as_bytes());
    let wrong_signature_b64 = STANDARD.encode(wrong_signature.to_bytes());

    let verify_result = refresh_service
        .verify_and_issue_access_token(
            &seed.refresh_token,
            seed.device_id,
            challenge.challenge_id,
            &wrong_signature_b64,
        )
        .await;
    assert!(matches!(verify_result, Err(AppError::Unauthorized(_))));

    let token_hash = RefreshService::hash_refresh_token(&seed.refresh_token);
    let revoked: bool = sqlx::query_scalar(
        r#"
        SELECT revoked
        FROM refresh_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_one(&ctx.db_pool)
    .await
    .unwrap();
    assert!(revoked, "invalid signature should revoke suspicious token");
}
