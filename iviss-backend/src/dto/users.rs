use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Agent,
    Manager,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Manager => "manager",
            Self::Agent => "agent",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Self::Admin,
            "manager" => Self::Manager,
            "agent" => Self::Agent,
            _ => Self::Agent,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    PendingActivation,
    Active,
    Suspended,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingActivation => "PENDING_ACTIVATION",
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "PENDING_ACTIVATION" => Self::PendingActivation,
            "ACTIVE" => Self::Active,
            "SUSPENDED" => Self::Suspended,
            _ => Self::PendingActivation,
        }
    }
}

/// Response for GET /users/me
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")] // ← converts to camelCase in JSON
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub organization_id: Uuid,
    pub organization: Option<String>,
    pub badge_id: Option<String>,
    pub phone_number: Option<String>,
    pub avatar_initials: Option<String>,
    pub status: UserStatus,
    pub is_active: bool,
}

/// Request for POST /admin/users (provisioning)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionUserRequest {
    pub username: String,
    pub phone_number: String,
    pub full_name: String,
    pub role: UserRole,
    pub organization_id: Uuid,
    pub email: Option<String>,
    pub badge_id: Option<String>,
}

/// Request for PATCH /admin/users/:id (update)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<UserRole>,
    pub organization_id: Option<Uuid>,
    pub email: Option<String>,
    pub badge_id: Option<String>,
    pub status: Option<UserStatus>,
}
