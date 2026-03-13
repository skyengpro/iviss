use serde::{Deserialize, Serialize};

use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResendActivationRequest {
    pub user_id: uuid::Uuid,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResendActivationResponse {
    pub message: String,
}

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
}

impl std::str::FromStr for UserRole {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "admin" => Self::Admin,
            "manager" => Self::Manager,
            "agent" => Self::Agent,
            _ => Self::Agent,
        })
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
}

impl std::str::FromStr for UserStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PENDING_ACTIVATION" => Self::PendingActivation,
            "ACTIVE" => Self::Active,
            "SUSPENDED" => Self::Suspended,
            _ => Self::PendingActivation,
        })
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_conversion() {
        assert_eq!(UserRole::Admin.as_str(), "admin");
        assert_eq!(UserRole::Manager.as_str(), "manager");
        assert_eq!(UserRole::Agent.as_str(), "agent");

        use std::str::FromStr;
        assert!(matches!(
            UserRole::from_str("admin").unwrap(),
            UserRole::Admin
        ));
        assert!(matches!(
            UserRole::from_str("ADMIN").unwrap(),
            UserRole::Admin
        ));
        assert!(matches!(
            UserRole::from_str("manager").unwrap(),
            UserRole::Manager
        ));
        assert!(matches!(
            UserRole::from_str("agent").unwrap(),
            UserRole::Agent
        ));
        assert!(matches!(
            UserRole::from_str("unknown").unwrap(),
            UserRole::Agent
        ));
    }

    #[test]
    fn test_user_status_conversion() {
        assert_eq!(UserStatus::Active.as_str(), "ACTIVE");
        assert_eq!(UserStatus::Suspended.as_str(), "SUSPENDED");
        assert_eq!(UserStatus::PendingActivation.as_str(), "PENDING_ACTIVATION");

        use std::str::FromStr;
        assert!(matches!(
            UserStatus::from_str("ACTIVE").unwrap(),
            UserStatus::Active
        ));
        assert!(matches!(
            UserStatus::from_str("SUSPENDED").unwrap(),
            UserStatus::Suspended
        ));
        assert!(matches!(
            UserStatus::from_str("PENDING_ACTIVATION").unwrap(),
            UserStatus::PendingActivation
        ));
        assert!(matches!(
            UserStatus::from_str("UNKNOWN").unwrap(),
            UserStatus::PendingActivation
        ));
    }
}
