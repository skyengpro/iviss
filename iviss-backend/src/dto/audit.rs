use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// The action types for audit logging, matching the PostgreSQL `audit_action` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "audit_action", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    // Auth
    LoginSuccess,
    LoginFailed,
    Logout,
    TokenRefreshed,
    OtpRequested,
    OtpVerified,
    OtpFailed,
    // Device
    DeviceRegistered,
    DeviceRevoked,
    DeviceSuspended,
    // User management
    UserCreated,
    UserUpdated,
    UserSuspended,
    UserActivated,
    UserDeleted,
    // Session
    SessionTerminated,
    SessionRestarted,
    ActivationCodeResent,
    // Vehicle control
    VehicleSearched,
    VehicleNotFound,
    PendingSubmissionCreated,
    PendingSubmissionReviewed,
}

impl FromStr for AuditAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOGIN_SUCCESS" => Ok(Self::LoginSuccess),
            "LOGIN_FAILED" => Ok(Self::LoginFailed),
            "LOGOUT" => Ok(Self::Logout),
            "TOKEN_REFRESHED" => Ok(Self::TokenRefreshed),
            "OTP_REQUESTED" => Ok(Self::OtpRequested),
            "OTP_VERIFIED" => Ok(Self::OtpVerified),
            "OTP_FAILED" => Ok(Self::OtpFailed),
            "DEVICE_REGISTERED" => Ok(Self::DeviceRegistered),
            "DEVICE_REVOKED" => Ok(Self::DeviceRevoked),
            "DEVICE_SUSPENDED" => Ok(Self::DeviceSuspended),
            "USER_CREATED" => Ok(Self::UserCreated),
            "USER_UPDATED" => Ok(Self::UserUpdated),
            "USER_SUSPENDED" => Ok(Self::UserSuspended),
            "USER_ACTIVATED" => Ok(Self::UserActivated),
            "USER_DELETED" => Ok(Self::UserDeleted),
            "SESSION_TERMINATED" => Ok(Self::SessionTerminated),
            "SESSION_RESTARTED" => Ok(Self::SessionRestarted),
            "ACTIVATION_CODE_RESENT" => Ok(Self::ActivationCodeResent),
            "VEHICLE_SEARCHED" => Ok(Self::VehicleSearched),
            "VEHICLE_NOT_FOUND" => Ok(Self::VehicleNotFound),
            "PENDING_SUBMISSION_CREATED" => Ok(Self::PendingSubmissionCreated),
            "PENDING_SUBMISSION_REVIEWED" => Ok(Self::PendingSubmissionReviewed),
            _ => Err(()),
        }
    }
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LoginSuccess => "LOGIN_SUCCESS",
            Self::LoginFailed => "LOGIN_FAILED",
            Self::Logout => "LOGOUT",
            Self::TokenRefreshed => "TOKEN_REFRESHED",
            Self::OtpRequested => "OTP_REQUESTED",
            Self::OtpVerified => "OTP_VERIFIED",
            Self::OtpFailed => "OTP_FAILED",
            Self::DeviceRegistered => "DEVICE_REGISTERED",
            Self::DeviceRevoked => "DEVICE_REVOKED",
            Self::DeviceSuspended => "DEVICE_SUSPENDED",
            Self::UserCreated => "USER_CREATED",
            Self::UserUpdated => "USER_UPDATED",
            Self::UserSuspended => "USER_SUSPENDED",
            Self::UserActivated => "USER_ACTIVATED",
            Self::UserDeleted => "USER_DELETED",
            Self::SessionTerminated => "SESSION_TERMINATED",
            Self::SessionRestarted => "SESSION_RESTARTED",
            Self::ActivationCodeResent => "ACTIVATION_CODE_RESENT",
            Self::VehicleSearched => "VEHICLE_SEARCHED",
            Self::VehicleNotFound => "VEHICLE_NOT_FOUND",
            Self::PendingSubmissionCreated => "PENDING_SUBMISSION_CREATED",
            Self::PendingSubmissionReviewed => "PENDING_SUBMISSION_REVIEWED",
        }
    }
}

/// A single audit log entry returned by the API.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: AuditAction,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub before_snapshot: Option<serde_json::Value>,
    pub after_snapshot: Option<serde_json::Value>,
    /// ISO 8601 formatted timestamp string
    pub created_at: String,
}

/// Paginated audit log response.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogListResponse {
    pub items: Vec<AuditLogEntry>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Query parameters for listing/exporting audit logs.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "snake_case")]
pub struct AuditLogQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}
