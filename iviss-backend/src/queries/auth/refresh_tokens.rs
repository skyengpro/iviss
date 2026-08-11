use crate::dto::users::{UserRole, UserStatus};
use crate::errors::{AppError, ErrorCode};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

pub async fn has_valid_refresh_token(pool: &PgPool, device_id: Uuid) -> Result<bool, AppError> {
    let valid_refresh: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM refresh_tokens
            WHERE  device_id  = $1
              AND  revoked    = FALSE
              AND  expires_at > NOW()
        )
        "#,
    )
    .bind(device_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(valid_refresh)
}

pub async fn insert_refresh_token(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token_hash: &str,
    user_id: Uuid,
    device_id: Uuid,
    expires_at: time::OffsetDateTime,
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
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_web_refresh_token(
    pool: &PgPool,
    token_hash: &str,
    user_id: Uuid,
    expires_at: time::OffsetDateTime,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(Option::<Uuid>::None)
    .bind(expires_at)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn revoke_active_refresh_tokens_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE, revoked_at = NOW()
        WHERE user_id = $1
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}

pub async fn insert_refresh_and_activate_device(
    pool: &PgPool,
    device_id: Uuid,
    token_hash: &str,
    user_id: Uuid,
    expires_at: time::OffsetDateTime,
    shift_start: i64,
    shift_end: i64,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
                WITH insert_refresh AS (
                    INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
                    VALUES ($2, $3, $1, $4)
                )
                UPDATE devices
                SET    status       = 'ACTIVE'::device_status,
                       metadata     = jsonb_build_object('shift_start', $5, 'shift_end', $6),
                       last_seen_at = NOW()
                WHERE  id = $1
                "#,
    )
    .bind(device_id)
    .bind(token_hash)
    .bind(user_id)
    .bind(expires_at)
    .bind(shift_start)
    .bind(shift_end)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn validate_agent_refresh_token(
    pool: &PgPool,
    token_hash: &str,
    device_id: Uuid,
) -> Result<bool, AppError> {
    let token_row = sqlx::query(
        r#"
        SELECT user_id, device_id
        FROM refresh_tokens
        WHERE token_hash = $1
          AND device_id = $2
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(token_row.is_some())
}

pub struct AdminRefreshRow {
    pub user_id: Uuid,
    pub role: UserRole,
    pub status: UserStatus,
}

pub async fn get_admin_refresh_context(
    pool: &PgPool,
    token_hash: &str,
) -> Result<AdminRefreshRow, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            rt.user_id,
            role,
            status
        FROM refresh_tokens rt
        JOIN users u ON u.id = rt.user_id
        WHERE rt.token_hash = $1
          AND rt.device_id IS NULL
          AND rt.revoked = FALSE
          AND rt.expires_at > NOW()
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "admin refresh: database error during token lookup");
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        tracing::warn!("admin refresh: FAILED — refresh token not found, revoked, or expired");
        AppError::Unauthorized("Invalid or expired refresh token".into())
    })?;

    Ok(AdminRefreshRow {
        user_id: row.get("user_id"),
        role: row.get("role"),
        status: row.get("status"),
    })
}

pub async fn get_refresh_token_user_id(
    pool: &PgPool,
    token_hash: &str,
    device_id: Uuid,
) -> Result<Uuid, AppError> {
    let token_row = sqlx::query(
        r#"
        SELECT user_id
        FROM refresh_tokens
        WHERE token_hash = $1
          AND device_id = $2
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| {
        tracing::warn!(device_id = %device_id, "Verification failed: Invalid or expired refresh token");
        AppError::unauthorized_with_code(ErrorCode::SessionRevoked, "Invalid or expired refresh token")
    })?;

    Ok(token_row.get("user_id"))
}
