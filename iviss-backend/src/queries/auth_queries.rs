use crate::dto::users::{UserRole, UserStatus};
use crate::errors::AppError;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

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
) -> Result<usize, AppError> {
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
    Ok(0)
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

/// Find admin/manager user by email for email/password login.
///
/// This function explicitly excludes agents - they cannot log in via email/password.
/// Only returns users with role 'admin' or 'manager'.
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
            organization_id,
            full_name,
            email,
            username,
            phone_number
        FROM users
        WHERE email = $1
          AND role IN ('admin', 'manager')
          AND deleted_at IS NULL
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    Ok(result)
}
