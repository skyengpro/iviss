use crate::dto::users::{UserProfile, UserRole};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<UserProfile, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            u.id, 
            u.full_name, 
            COALESCE(u.email, '') AS email,
            u.role::text AS role,
            u.organization_id, 
            o.name as organization_name,
            u.badge_id,
            u.phone_number,
            u.status::text AS status
        FROM users u
        JOIN organizations o ON u.organization_id = o.id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("User not found"))?;

    let role_str: String = row.get("role");
    let role = match role_str.as_str() {
        "admin" => UserRole::Admin,
        "manager" => UserRole::Manager,
        "agent" => UserRole::Agent,
        _ => UserRole::Agent, // Default fallback
    };

    Ok(UserProfile {
        id: row.get("id"),
        name: row.get("full_name"),
        email: row.get("email"),
        role,
        organization_id: row.get("organization_id"),
        organization: row.get("organization_name"),
        badge_id: row.get("badge_id"),
        phone_number: row.get("phone_number"),
        avatar_initials: None, // Derived field maybe?
        is_active: row.get::<String, _>("status") == "ACTIVE",
    })
}
