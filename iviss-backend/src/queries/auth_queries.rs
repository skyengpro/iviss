use crate::errors::AppError;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;
use deadpool_redis::redis::AsyncCommands;
use crate::db::RedisPool;
use time::PrimitiveDateTime;

#[derive(Debug, FromRow)]
pub struct AuthValidationContext {
    pub is_blacklisted: bool,
    pub user_status: Option<String>,
    pub device_is_active: bool,
}

pub async fn get_auth_validation_context(
    pool: &PgPool,
    user_id: Uuid,
    device_id: Uuid,
    jti: &str,
) -> Result<AuthValidationContext, AppError> {
    sqlx::query_as::<_, AuthValidationContext>(
        r#"
        SELECT
            EXISTS(
                SELECT 1
                FROM access_token_blacklist atb
                WHERE atb.jti = $3
                  AND atb.expires_at > NOW()
            ) AS is_blacklisted,
            (
                SELECT u.status::text
                FROM users u
                WHERE u.id = $1
                  AND u.deleted_at IS NULL
            ) AS user_status,
            EXISTS(
                SELECT 1
                FROM devices d
                WHERE d.id = $2
                  AND d.user_id = $1
                  AND d.status = 'ACTIVE'::device_status
            ) AS device_is_active
        "#,
    )
    .bind(user_id)
    .bind(device_id)
    .bind(jti)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub async fn mark_device_inactive(pool: &PgPool, device_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE devices
        SET status = 'INACTIVE'::device_status,
            revoked_at = NOW()
        WHERE id = $1
          AND status = 'ACTIVE'::device_status
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}


#[derive(Debug, FromRow)]
pub struct UserForLogin {
    pub id: Uuid,
    pub role: String,
    pub status: String,
    pub badge_id: Option<String>,
    pub phone_number: String,
}

pub async fn get_user_by_phone(
    pool: &PgPool,
    phone_number: &str,
) -> Result<UserForLogin, AppError> {
    sqlx::query_as::<_, UserForLogin>(
        r#"
        SELECT id, role::TEXT AS role, status::TEXT AS status,
               badge_id, phone_number
        FROM users
        WHERE phone_number = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(phone_number)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("User not found"))
}

pub async fn get_user_by_badge_id(
    pool: &PgPool,
    badge_id: &str,
) -> Result<UserForLogin, AppError> {
    sqlx::query_as::<_, UserForLogin>(
        r#"
        SELECT id, role::TEXT AS role, status::TEXT AS status,
               badge_id, phone_number
        FROM users
        WHERE badge_id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(badge_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("User not found"))
}

#[derive(Debug, FromRow)]
pub struct DeviceForLogin {
    pub id: Uuid,
    pub status: String,
}

pub async fn get_device_by_user(
    pool: &PgPool,
    device_id: Uuid,
    user_id: Uuid,
) -> Result<DeviceForLogin, AppError> {
    sqlx::query_as::<_, DeviceForLogin>(
        r#"
        SELECT id, status::TEXT AS status
        FROM devices
        WHERE id = $1
          AND user_id = $2
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("Device not found"))
}

pub async fn mark_device_active(
    pool: &PgPool,
    device_id: Uuid,
    shift_start: i64,
    shift_end: i64,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE devices
        SET status       = 'ACTIVE'::device_status,
            metadata     = jsonb_build_object('shift_start', $2, 'shift_end', $3),
            last_seen_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(device_id)
    .bind(shift_start)
    .bind(shift_end)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}

pub async fn mark_device_suspended(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE devices
        SET status     = 'SUSPENDED'::device_status,
            revoked_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}

pub async fn store_refresh_token(
    pool: &PgPool,
    token_hash: &str,
    user_id: Uuid,
    device_id: Uuid,
    expires_at: PrimitiveDateTime,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(device_id)
    .bind(expires_at)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}

pub async fn revoke_refresh_tokens_for_device(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked    = TRUE,
            revoked_at = NOW()
        WHERE device_id = $1
          AND revoked   = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}

pub async fn blacklist_jti(
    redis: &RedisPool,
    jti: &str,
    ttl_secs: u64,
) -> Result<(), AppError> {
    let key = format!("blacklist:jti:{}", jti);
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::internal_error(format!("Redis connection failed: {e}")))?;

    conn.set_ex::<_, _, ()>(&key, "1", ttl_secs)
        .await
        .map_err(|e| AppError::internal_error(format!("Redis SET failed: {e}")))?;

    Ok(())
}