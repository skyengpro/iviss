use crate::dto::users::{DeviceStatus, UserProfile, UserRole, UserStatus};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub mod location;

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<UserProfile, AppError> {
    // 1. Optimisation SQL avec LATERAL
    let row = sqlx::query(
        r#"
        SELECT 
            u.id, 
            u.full_name, 
            u.email, 
            u.role,      
            u.organization_id, 
            o.name AS organization_name,
            u.badge_id,
            u.phone_number,
            u.status,
            u.username,
            d.status AS session_status,
            d.revoked_at AS last_revoked_at
        FROM users u
        LEFT JOIN organizations o ON u.organization_id = o.id
        LEFT JOIN (
            SELECT DISTINCT ON (user_id)
                user_id, status, revoked_at
            FROM devices
            ORDER BY user_id, updated_at DESC
        ) d ON u.id = d.user_id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("User not found"))?;

    let role: UserRole = row.get("role");
    let status: UserStatus = row.get("status");

    let session_status: Option<DeviceStatus> = row.get("session_status");

    Ok(UserProfile {
        id: row.get("id"),
        username: row.get("username"),
        name: row.get("full_name"),
        email: row.get("email"),
        role,
        organization_id: row.get("organization_id"),
        organization: row.get("organization_name"),
        badge_id: row.get("badge_id"),
        phone_number: row.get("phone_number"),
        avatar_initials: None,
        is_active: status == UserStatus::Active,
        status,
        session_status,
        last_revoked_at: row.get("last_revoked_at"),
    })
}

pub async fn create_user(
    pool: &PgPool,
    req: crate::dto::users::ProvisionUserRequest,
) -> Result<UserProfile, AppError> {
    let role_str = req.role.as_str();

    let user_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (
            id, 
            organization_id, 
            username, 
            email, 
            role, 
            badge_id, 
            full_name, 
            phone_number, 
            status
        )
        VALUES ($1, $2, $3, $4, $5::user_role, $6, $7, $8, 'PENDING_ACTIVATION'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(req.organization_id)
    .bind(req.username)
    .bind(req.email)
    .bind(role_str)
    .bind(req.badge_id)
    .bind(req.full_name)
    .bind(req.phone_number)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    get_user_by_id(pool, user_id).await
}

pub async fn create_org_admin_user_with_temp_password(
    pool: &PgPool,
    req: crate::dto::users::ProvisionUserRequest,
    password_hash: String,
) -> Result<UserProfile, AppError> {
    let user_id = Uuid::new_v4();

    let email = req
        .email
        .ok_or_else(|| AppError::bad_request("email is required for org admin"))?;

    sqlx::query(
        r#"
        INSERT INTO users (
            id,
            organization_id,
            username,
            email,
            password_hash,
            must_change_password,
            role,
            badge_id,
            full_name,
            phone_number,
            status
        )
        VALUES (
            $1, $2, $3, $4, $5, TRUE,
            'org_admin'::user_role,
            $6, $7, $8,
            'PENDING_ACTIVATION'::user_status
        )
        "#,
    )
    .bind(user_id)
    .bind(req.organization_id)
    .bind(req.username)
    .bind(email)
    .bind(password_hash)
    .bind(req.badge_id)
    .bind(req.full_name)
    .bind(req.phone_number)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    get_user_by_id(pool, user_id).await
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<UserProfile>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            u.id, 
            u.full_name, 
            u.email, 
            u.role AS role,
            u.organization_id, 
            o.name AS organization_name,
            u.badge_id,
            u.phone_number,
            u.status,
            u.username,
            d.status AS session_status,
            d.revoked_at AS last_revoked_at
        FROM users u
        LEFT JOIN organizations o ON u.organization_id = o.id
        LEFT JOIN (
            SELECT DISTINCT ON (user_id)
                user_id, status, revoked_at
            FROM devices
            ORDER BY user_id, updated_at DESC
        ) d ON u.id = d.user_id
        WHERE u.deleted_at IS NULL
        ORDER BY u.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let users = rows
        .into_iter()
        .map(|row| {
            let role: UserRole = row.get("role");
            let status: UserStatus = row.get("status");

            let session_status: Option<DeviceStatus> = row.get("session_status");

            UserProfile {
                id: row.get("id"),
                username: row.get("username"),
                name: row.get("full_name"),
                email: row.get("email"),
                role,
                organization_id: row.get("organization_id"),
                organization: row.get("organization_name"),
                badge_id: row.get("badge_id"),
                phone_number: row.get("phone_number"),
                avatar_initials: None,
                is_active: status == UserStatus::Active,
                status,
                session_status,
                last_revoked_at: row.get("last_revoked_at"),
            }
        })
        .collect();

    Ok(users)
}

pub async fn list_users_by_org(pool: &PgPool, org_id: Uuid) -> Result<Vec<UserProfile>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            u.id, 
            u.full_name, 
            u.email, 
            u.role AS role,
            u.organization_id, 
            o.name AS organization_name,
            u.badge_id,
            u.phone_number,
            u.status,
            u.username,
            d.status AS session_status,
            d.revoked_at AS last_revoked_at
        FROM users u
        LEFT JOIN organizations o ON u.organization_id = o.id
        LEFT JOIN (
            SELECT DISTINCT ON (user_id)
                user_id, status, revoked_at
            FROM devices
            ORDER BY user_id, updated_at DESC
        ) d ON u.id = d.user_id
        WHERE u.deleted_at IS NULL
          AND u.organization_id = $1
        ORDER BY u.created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let users = rows
        .into_iter()
        .map(|row| {
            let role: UserRole = row.get("role");
            let status: UserStatus = row.get("status");
            let session_status: Option<DeviceStatus> = row.get("session_status");
            UserProfile {
                id: row.get("id"),
                username: row.get("username"),
                name: row.get("full_name"),
                email: row.get("email"),
                role,
                organization_id: row.get("organization_id"),
                organization: row.get("organization_name"),
                badge_id: row.get("badge_id"),
                phone_number: row.get("phone_number"),
                avatar_initials: None,
                is_active: status == UserStatus::Active,
                status,
                session_status,
                last_revoked_at: row.get("last_revoked_at"),
            }
        })
        .collect();

    Ok(users)
}

pub async fn update_user(
    pool: &PgPool,
    user_id: Uuid,
    req: crate::dto::users::UpdateUserRequest,
) -> Result<UserProfile, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("UPDATE users SET ");
    let mut separated = query_builder.separated(", ");

    if let Some(username) = req.username {
        separated
            .push("username = ")
            .push_bind_unseparated(username);
    }
    if let Some(phone_number) = req.phone_number {
        separated
            .push("phone_number = ")
            .push_bind_unseparated(phone_number);
    }
    if let Some(full_name) = req.full_name {
        separated
            .push("full_name = ")
            .push_bind_unseparated(full_name);
    }
    if let Some(role) = req.role {
        separated
            .push("role = ")
            .push_bind_unseparated(role.as_str())
            .push_unseparated("::user_role");
    }
    if let Some(org_id) = req.organization_id {
        separated
            .push("organization_id = ")
            .push_bind_unseparated(org_id);
    }
    if let Some(email) = req.email {
        separated.push("email = ").push_bind_unseparated(email);
    }
    if let Some(badge_id) = req.badge_id {
        separated
            .push("badge_id = ")
            .push_bind_unseparated(badge_id);
    }
    let is_suspended = req.status == Some(UserStatus::Suspended);
    let is_pending = req.status == Some(UserStatus::PendingActivation);
    if let Some(status) = req.status {
        separated
            .push("status = ")
            .push_bind_unseparated(status.as_str())
            .push_unseparated("::user_status");
    }

    query_builder.push(" WHERE id = ").push_bind(user_id);
    query_builder.push(" AND deleted_at IS NULL");

    query_builder
        .build()
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    if is_pending {
        // Set the user device pending
        sqlx::query(
            r#"
            UPDATE devices
            SET status = 'PENDING'::device_status
            WHERE user_id = $1
              AND status = 'SUSPENDED'::device_status
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    }

    if is_suspended {
        // Suspend all active devices for this user
        sqlx::query(
            r#"
            UPDATE devices
            SET status = 'SUSPENDED'::device_status,
                revoked_at = NOW()
            WHERE user_id = $1
              AND status = 'ACTIVE'::device_status
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

        // Revoke all active refresh tokens for this user
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
        .map_err(AppError::database)?;
    }

    tx.commit().await.map_err(AppError::database)?;

    get_user_by_id(pool, user_id).await
}

pub async fn hard_delete_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    // 1. Delete control actions linked to this user's controls
    sqlx::query(
        "DELETE FROM control_actions WHERE control_id IN (SELECT id FROM control_records WHERE agent_id = $1)"
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    // 2. Delete control records where user is agent
    sqlx::query("DELETE FROM control_records WHERE agent_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    // 3. Delete pending submissions where user is agent
    sqlx::query("DELETE FROM pending_submissions WHERE agent_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    // 4. Update pending submissions where user was reviewer (set to null instead of delete?)
    // Actually user wants permanent delete of ALL related data.
    // If we delete the submission, it's safer.
    sqlx::query("DELETE FROM pending_submissions WHERE reviewed_by = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    // 5. Delete the user
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;

    Ok(())
}

pub struct ActivationResendUserRow {
    pub id: Uuid,
    pub phone_number: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub organization_id: Option<Uuid>,
    pub device_status: Option<DeviceStatus>,
}

pub async fn get_activation_resend_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<ActivationResendUserRow, AppError> {
    let user_raw = sqlx::query(
        r#"
        SELECT u.id,
               u.phone_number,
               u.role,
               u.status,
               u.organization_id,
               d.status AS device_status
        FROM users u
        LEFT JOIN (
            SELECT DISTINCT ON (user_id)
                user_id, status
            FROM devices
            ORDER BY user_id, updated_at DESC
        ) d ON u.id = d.user_id
        WHERE u.id = $1
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(ActivationResendUserRow {
        id: user_raw.get("id"),
        phone_number: user_raw.get("phone_number"),
        role: user_raw.get("role"),
        status: user_raw.get("status"),
        organization_id: user_raw.get("organization_id"),
        device_status: user_raw.get("device_status"),
    })
}

pub async fn mark_user_pending_and_revoke_refresh_tokens(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    sqlx::query(
        r#"
            UPDATE users
            SET status = 'PENDING_ACTIVATION'::user_status
            WHERE id = $1
              AND deleted_at IS NULL
            "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

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

    tx.commit().await.map_err(AppError::Database)
}

pub struct OrgAdminPasswordResendRow {
    pub id: Uuid,
    pub email: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
}

pub async fn get_org_admin_password_resend_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<OrgAdminPasswordResendRow, AppError> {
    let user_raw = sqlx::query(
        r#"
        SELECT id, email, role, status
        FROM users
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(OrgAdminPasswordResendRow {
        id: user_raw.get("id"),
        email: user_raw.get("email"),
        role: user_raw.get("role"),
        status: user_raw.get("status"),
    })
}

pub async fn update_org_admin_temporary_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1,
            must_change_password = TRUE
        WHERE id = $2
          AND deleted_at IS NULL
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}
