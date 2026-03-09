use crate::errors::AppError;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

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
