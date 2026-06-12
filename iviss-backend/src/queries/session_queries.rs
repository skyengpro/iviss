use crate::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

/// Terminates all sessions for a given user within a single transaction:
/// 1. Revokes all refresh tokens.
/// 2. Marks all devices as REVOKED.
///
/// The auth middleware already rejects requests when `device_is_active = false`,
/// so the very next request from the terminated user will return 401.
/// The user's account status is left unchanged.
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

    // 2. Mark all active devices as REVOKED
    sqlx::query(
        r#"
        UPDATE devices
        SET status = 'REVOKED'::device_status,
            revoked_at = NOW()
        WHERE user_id = $1
          AND status = 'ACTIVE'::device_status
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

/// Reactivates the most recent device for a user and extends its shift.
/// This allows an agent to resume work without a full OTP re-activation.
pub async fn restart_user_session(
    pool: &PgPool,
    user_id: Uuid,
    shift_duration: std::time::Duration,
) -> Result<(), AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time before UNIX_EPOCH"))?
        .as_secs();

    let shift_start: i64 = now.try_into().unwrap_or(0);
    let shift_end: i64 = now
        .saturating_add(shift_duration.as_secs())
        .try_into()
        .unwrap_or(0);

    // Update the most recently updated device for this user to PENDING, clearing revoked_at
    sqlx::query(
        r#"
        UPDATE devices
        SET status = 'PENDING'::device_status,
            revoked_at = NULL,
            metadata = metadata || jsonb_build_object('shift_start', $2, 'shift_end', $3)
        WHERE id = (
            SELECT id FROM devices 
            WHERE user_id = $1 
            ORDER BY updated_at DESC 
            LIMIT 1
        )
        "#,
    )
    .bind(user_id)
    .bind(shift_start)
    .bind(shift_end)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    tracing::info!(%user_id, "session: session restarted for user");

    Ok(())
}
