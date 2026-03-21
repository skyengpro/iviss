use crate::dto::users::{UserProfile, UserRole, UserStatus};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<UserProfile, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            u.id, 
            u.full_name, 
            u.email, 
            u.role::TEXT as role, 
            u.organization_id, 
            o.name as organization_name,
            u.badge_id,
            u.phone_number,
            u.status::TEXT as status,
            u.username
        FROM users u
        LEFT JOIN organizations o ON u.organization_id = o.id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("User not found"))?;

    let role_str: String = row.get("role");
    let role = role_str.parse::<UserRole>().map_err(|_| {
        tracing::error!(role = %role_str, "Unknown role in DB");
        AppError::internal_error("Invalid user role in database")
    })?;

    let status_str: String = row.get("status");
    let status = status_str
        .parse::<crate::dto::users::UserStatus>()
        .map_err(|_| {
            tracing::error!(status = %status_str, "Unknown status in DB");
            AppError::internal_error("Invalid user status in database")
        })?;

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
        avatar_initials: None, // Derived field maybe?
        is_active: status_str == "ACTIVE",
        status,
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

pub async fn list_users(pool: &PgPool) -> Result<Vec<UserProfile>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            u.id, 
            u.full_name, 
            u.email, 
            u.role::TEXT as role, 
            u.organization_id, 
            o.name as organization_name,
            u.badge_id,
            u.phone_number,
            u.status::TEXT as status,
            u.username
        FROM users u
        LEFT JOIN organizations o ON u.organization_id = o.id
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
        .execute(pool)
        .await
        .map_err(AppError::database)?;

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
