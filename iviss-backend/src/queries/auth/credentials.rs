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
    pub organization_id: Option<Uuid>,
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
            (
                SELECT u.organization_id
                FROM users u
                WHERE u.id = $1
                  AND u.deleted_at IS NULL
            ) AS organization_id,
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
