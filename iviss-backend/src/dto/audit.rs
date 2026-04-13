use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// The action types for audit logging, matching the PostgreSQL `audit_action` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "audit_action", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
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
