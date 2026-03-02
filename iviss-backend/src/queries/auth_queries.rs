use crate::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn is_token_blacklisted(pool: &PgPool, jti: &str) -> Result<bool, AppError> {
    let is_blacklisted = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM access_token_blacklist
            WHERE jti = $1
              AND expires_at > NOW()
        )
        "#,
    )
    .bind(jti)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(is_blacklisted)
}

pub async fn get_user_status(pool: &PgPool, user_id: Uuid) -> Result<Option<String>, AppError> {
    let status = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status::text
        FROM users
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    Ok(status)
}

pub async fn is_device_active_for_user(
    pool: &PgPool,
    device_id: Uuid,
    user_id: Uuid,
) -> Result<bool, AppError> {
    let is_active = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM devices
            WHERE id = $1
              AND user_id = $2
              AND status = 'ACTIVE'::device_status
        )
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(is_active)
}
