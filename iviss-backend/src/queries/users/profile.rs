use crate::dto::users::{DeviceStatus, UserProfile, UserRole, UserStatus};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
