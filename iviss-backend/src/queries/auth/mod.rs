use crate::dto::users::{UserRole, UserStatus};
use crate::errors::{AppError, ErrorCode};
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

pub mod sessions;

#[derive(Debug, FromRow)]
pub struct AuthValidationContext {
    pub is_blacklisted: bool,
    pub user_status: Option<String>,
    pub device_is_active: bool,
}

/// Admin authentication row - contains credentials for email/password login
#[derive(Debug, FromRow)]
pub struct AdminAuthRow {
    pub id: Uuid,
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub must_change_password: bool,
    pub organization_id: Option<Uuid>,
    pub full_name: String,
    pub email: String,
    pub username: String,
    pub phone_number: String,
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

#[derive(Debug, FromRow)]
pub struct UserForLogin {
    pub id: Uuid,
    pub role: String,
    pub status: String,
    pub phone_number: String,
}

pub async fn get_user_by_badge(pool: &PgPool, badge_id: &str) -> Result<UserForLogin, AppError> {
    sqlx::query_as::<_, UserForLogin>(
        r#"
        SELECT id, role::TEXT AS role, status::TEXT AS status,
               phone_number
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

/// Blacklist a JTI in PostgreSQL for persistence
pub async fn blacklist_jti_db(
    pool: &PgPool,
    jti: &str,
    user_id: Uuid,
    expires_at: time::OffsetDateTime,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO access_token_blacklist (jti, user_id, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (jti) DO NOTHING
        "#,
    )
    .bind(jti)
    .bind(user_id)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(())
}

/// Load active blacklisted JTIs from PostgreSQL into cache (background task, limited to cache capacity)
pub async fn load_blacklisted_jtis_to_cache(
    pool: &PgPool,
    cache: &crate::app_cache::AppCache,
) -> Result<(), AppError> {
    let cache_clone = cache.clone();
    let pool_clone = pool.clone();

    // Spawn background task to avoid blocking startup
    // Limit to 10000 (Moka cache max_capacity) to avoid wasting memory
    tokio::spawn(async move {
        let rows = sqlx::query(
            r#"
            SELECT jti
            FROM access_token_blacklist
            WHERE expires_at > NOW()
            ORDER BY expires_at DESC
            LIMIT 10000
            "#,
        )
        .fetch_all(&pool_clone)
        .await;

        match rows {
            Ok(rows) => {
                let count = rows.len();

                for row in rows {
                    let jti: String = row.get("jti");
                    cache_clone.jti_blacklist.insert(jti, ()).await;
                }

                tracing::info!(
                    count,
                    "Loaded blacklisted JTIs from PostgreSQL to cache (background task completed)"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to load blacklisted JTIs from PostgreSQL (background task)");
            }
        }
    });

    tracing::info!("Blacklisted JTIs loading started in background (max 10000, most recent first)");
    Ok(())
}

/// Blacklist a JTI in the Moka cache
pub async fn blacklist_jti_cache(
    cache: &crate::app_cache::AppCache,
    jti: &str,
) -> Result<(), AppError> {
    cache.jti_blacklist.insert(jti.to_string(), ()).await;
    Ok(())
}

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

/// Find admin/manager/org_admin user by email for login.
///
/// This function explicitly excludes agents - they cannot log in via email/password.
/// Only returns users with role 'admin', 'manager', or 'org_admin'.
pub async fn find_admin_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<AdminAuthRow>, AppError> {
    let result = sqlx::query_as::<_, AdminAuthRow>(
        r#"
        SELECT 
            id,
            password_hash,
            role,
            status,
            must_change_password,
            organization_id,
            full_name,
            email,
            username,
            phone_number
        FROM users
        WHERE email = $1
          AND role IN ('admin', 'manager', 'org_admin')
          AND deleted_at IS NULL
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    Ok(result)
}

/// Find admin/manager/org_admin user by either email or username.
pub async fn find_admin_by_identity(
    pool: &PgPool,
    identity: &str,
) -> Result<Option<AdminAuthRow>, AppError> {
    let result = sqlx::query_as::<_, AdminAuthRow>(
        r#"
        SELECT 
            id,
            password_hash,
            role,
            status,
            must_change_password,
            organization_id,
            full_name,
            email,
            username,
            phone_number
        FROM users
        WHERE (email = $1 OR username = $1)
          AND role IN ('admin', 'manager', 'org_admin')
          AND deleted_at IS NULL
        "#,
    )
    .bind(identity)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    Ok(result)
}

pub async fn get_user_org_id(pool: &PgPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar(
        r#"
        SELECT organization_id
        FROM users
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
    .map(Option::flatten)
}

pub struct ActivationUserRow {
    pub id: Uuid,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub status: String,
}

pub async fn get_activation_user_by_badge(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    badge_id: &str,
) -> Result<ActivationUserRow, AppError> {
    let user_row = sqlx::query(
        r#"
        SELECT id,
               role,
               organization_id,
               status::TEXT AS status
        FROM users
        WHERE badge_id = $1
        AND deleted_at IS NULL
        "#,
    )
    .bind(badge_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(ActivationUserRow {
        id: user_row.get("id"),
        role: user_row.get("role"),
        organization_id: user_row.get("organization_id"),
        status: user_row.get("status"),
    })
}

pub async fn mark_user_active(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users
        SET status = 'ACTIVE'::user_status
        WHERE id = $1
        AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
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

pub struct DailyLoginContextRow {
    pub user_id: Uuid,
    pub user_role: UserRole,
    pub user_status: UserStatus,
    pub device_status: String,
}

pub async fn get_daily_login_context(
    pool: &PgPool,
    badge_id: &str,
    device_id: Uuid,
) -> Result<DailyLoginContextRow, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            u.id              AS user_id,
            u.role            AS user_role,
            u.status          AS user_status,
            COALESCE(d.status::TEXT, 'INACTIVE') AS device_status
        FROM users u
        LEFT JOIN devices d
            ON d.user_id = u.id
           AND d.id      = $2
        WHERE u.badge_id    = $1
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(badge_id)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::not_found("User or device not found"))?;

    Ok(DailyLoginContextRow {
        user_id: row.get("user_id"),
        user_role: row.get("user_role"),
        user_status: row.get("user_status"),
        device_status: row.get("device_status"),
    })
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
            ErrorCode::DeviceReactivation,
            "Device not found or revoked",
        )
    })?;

    Ok(ActiveDeviceKeyMetadata {
        public_key: device_row.get("public_key"),
        metadata: device_row.get("metadata"),
    })
}

pub async fn get_password_change_state(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(String, bool), AppError> {
    sqlx::query_as(
        "SELECT password_hash, must_change_password FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_password_after_change(
    pool: &PgPool,
    user_id: Uuid,
    new_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1, must_change_password = FALSE, status = 'ACTIVE'::user_status
        WHERE id = $2
        "#,
    )
    .bind(new_hash)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn get_web_user_identity(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(Option<Uuid>, String), AppError> {
    let row = sqlx::query(
        "SELECT organization_id, email FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let (org_id, email): (Option<Uuid>, Option<String>) = row
        .map(|r| (r.get("organization_id"), r.get("email")))
        .ok_or_else(|| AppError::not_found("User not found"))?;

    Ok((org_id, email.unwrap_or_default()))
}
