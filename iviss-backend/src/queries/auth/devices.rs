use crate::errors::AppError;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

pub async fn mark_device_inactive(pool: &PgPool, device_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE devices
        SET status = 'INACTIVE'::device_status
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

#[derive(Debug, sqlx::FromRow)]
pub struct DeviceForLogin {
    pub status: String,
    pub revoked_at: Option<time::OffsetDateTime>,
}

pub async fn get_device_by_user_optional(
    pool: &PgPool,
    device_id: Uuid,
    user_id: Uuid,
) -> Result<Option<DeviceForLogin>, AppError> {
    sqlx::query_as::<_, DeviceForLogin>(
        r#"
        SELECT status::TEXT AS status, revoked_at
        FROM devices
        WHERE id = $1
          AND user_id = $2
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub async fn check_device_exists(pool: &PgPool, device_id: Uuid) -> Result<bool, AppError> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM devices WHERE id = $1
        )
        "#,
    )
    .bind(device_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(exists)
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
                SET    status       = 'ACTIVE'::device_status,
                       metadata     = jsonb_build_object('shift_start', $2, 'shift_end', $3),
                       last_seen_at = NOW()
                WHERE  id = $1
                "#,
    )
    .bind(device_id) // $1
    .bind(shift_start) // $2
    .bind(shift_end) // $3
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn suspend_device_and_revoke_tokens(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        WITH suspended_device AS (
            UPDATE devices
            SET    status     = 'SUSPENDED'::device_status,
                   revoked_at = NOW()
            WHERE  id = $1
        )
        UPDATE refresh_tokens
        SET    revoked    = TRUE,
               revoked_at = NOW()
        WHERE  device_id = $1
          AND  revoked   = FALSE
          AND  expires_at > NOW()
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}

pub async fn upsert_active_device(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    device_id: Uuid,
    user_id: Uuid,
    public_key_base64: &str,
    shift_start: i64,
    shift_end: i64,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO devices (id, user_id, public_key, status, metadata)
        VALUES (
            $1,
            $2,
            $3,
            'ACTIVE'::device_status,
            jsonb_build_object('shift_start', $4, 'shift_end', $5)
        )
        ON CONFLICT (id)
        DO UPDATE SET
            user_id = EXCLUDED.user_id,
            public_key = EXCLUDED.public_key,
            status = 'ACTIVE'::device_status,
            metadata = EXCLUDED.metadata,
            revoked_at = NULL
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .bind(public_key_base64)
    .bind(shift_start)
    .bind(shift_end)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn is_registered_unsuspended_device(
    pool: &PgPool,
    device_id: Uuid,
    user_id: Uuid,
) -> Result<bool, AppError> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM devices
            WHERE id = $1
              AND user_id = $2
              AND suspended_at IS NULL
        )
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub struct ActiveDeviceKeyMetadata {
    pub public_key: String,
    pub metadata: serde_json::Value,
}

pub async fn get_active_device_key_metadata(
    pool: &PgPool,
    device_id: Uuid,
    user_id: Uuid,
) -> Result<ActiveDeviceKeyMetadata, AppError> {
    let device_row = sqlx::query(
        r#"
        SELECT public_key, metadata
        FROM devices
        WHERE id = $1
          AND user_id = $2
          AND status = 'ACTIVE'::device_status
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| {
        tracing::warn!(device_id = %device_id, "Verification failed: Device not found or revoked");
        AppError::unauthorized_with_code(
            crate::errors::ErrorCode::DeviceReactivation,
            "Device not found or revoked",
        )
    })?;

    Ok(ActiveDeviceKeyMetadata {
        public_key: device_row.get("public_key"),
        metadata: device_row.get("metadata"),
    })
}
