use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
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

    // User management
    UserCreated,
    UserUpdated,
    UserSuspended,
    UserActivated,

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
            "USER_CREATED" => Ok(Self::UserCreated),
            "USER_UPDATED" => Ok(Self::UserUpdated),
            "USER_SUSPENDED" => Ok(Self::UserSuspended),
            "USER_ACTIVATED" => Ok(Self::UserActivated),
            "VEHICLE_SEARCHED" => Ok(Self::VehicleSearched),
            "VEHICLE_NOT_FOUND" => Ok(Self::VehicleNotFound),
            "PENDING_SUBMISSION_CREATED" => Ok(Self::PendingSubmissionCreated),
            "PENDING_SUBMISSION_REVIEWED" => Ok(Self::PendingSubmissionReviewed),
            _ => Err(()),
        }
    }
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
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
            Self::UserCreated => "USER_CREATED",
            Self::UserUpdated => "USER_UPDATED",
            Self::UserSuspended => "USER_SUSPENDED",
            Self::UserActivated => "USER_ACTIVATED",
            Self::VehicleSearched => "VEHICLE_SEARCHED",
            Self::VehicleNotFound => "VEHICLE_NOT_FOUND",
            Self::PendingSubmissionCreated => "PENDING_SUBMISSION_CREATED",
            Self::PendingSubmissionReviewed => "PENDING_SUBMISSION_REVIEWED",
        }
    }
}

/// DTO for an audit log entry
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub device_id: Option<Uuid>,
    pub action: AuditAction,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

/// Query parameters for listing audit logs
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogQuery {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
