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
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::redis::Redis;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "be06-refresh-jwt-secret-must-be-at-least-32-chars";

struct TestContext {
    _pg_container: ContainerAsync<GenericImage>,
    _redis_container: ContainerAsync<Redis>,
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
    let pg_image = GenericImage::new("postgres", "15-alpine")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "iviss")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ));
    let pg_container = pg_image.start().await.unwrap();
    let pg_port = pg_container.get_host_port_ipv4(5432).await.unwrap();

    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{pg_port}/iviss");
    let db_pool = PgPool::connect(&database_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();

    let redis_container = Redis::default().start().await.unwrap();
    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://127.0.0.1:{redis_port}");
    let redis_pool = RedisConfig::from_url(redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    TestContext {
        _pg_container: pg_container,
        _redis_container: redis_container,
        db_pool,
        redis_pool,
    }
}

async fn seed_active_refresh_token(pool: &PgPool) -> SeedData {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let refresh_token = format!("refresh-{}", Uuid::new_v4());
    let refresh_token_hash = RefreshService::hash_refresh_token(&refresh_token);

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
            $1, $2, 'be06-agent', NULL, NULL, 'agent'::user_role, 'AG-BE06', 'BE06 Agent', '+237600123456', 'ACTIVE'::user_status, NULL
        )
        "#,
    )
    .bind(user_id)
    .bind(organization_id)
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
