use crate::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

/// Terminates all sessions for a given user within a single transaction:
/// 1. Revokes all refresh tokens.
/// 2. Marks all devices as INACTIVE.
/// 3. Sets user status to SUSPENDED.
///
/// The auth middleware already rejects requests when `device_is_active = false`
/// or when user status is not ACTIVE, so the very next request from the 
/// terminated user will return 401.
pub async fn terminate_user_sessions(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // 1. Revoke all active refresh tokens for this user
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE,
            revoked_at = NOW()
        WHERE user_id = $1
          AND revoked = FALSE
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    // 2. Mark all active devices as INACTIVE
    sqlx::query(
        r#"
        UPDATE devices
        SET status = 'INACTIVE'::device_status,
            revoked_at = NOW()
        WHERE user_id = $1
          AND status = 'ACTIVE'::device_status
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    // 3. Suspend the user account
    sqlx::query(
        r#"
        UPDATE users
        SET status = 'SUSPENDED'::user_status
        WHERE id = $1
          AND status = 'ACTIVE'::user_status
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    tracing::info!(%user_id, "session: all sessions terminated for user");

    Ok(())
}
