use crate::dto::common::SubmissionLocation;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ── Request DTOs ──────────────────────────────────────────────────────────────

/// Request payload for creating a new pending submission
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePendingSubmissionRequest {
    pub plate_number: String,
    pub agent_id: Uuid,
    #[serde(alias = "frontImage")]
    pub front_image_url: String,
    #[serde(alias = "backImage")]
    pub back_image_url: String,
    pub notes: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Admin review action on a pending submission
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSubmissionRequest {
    /// Must be "approved" or "rejected"
    pub decision: SubmissionStatus,
    /// Required when decision is "rejected"
    pub rejection_reason: Option<String>,
    /// Required when decision is "approved" — the vehicle details to persist
    pub vehicle_data: Option<VehicleDataEntry>,
}

/// Vehicle details entered by the admin during approval
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VehicleDataEntry {
    pub chassis_number: String,
    pub brand: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
    pub engine_power: Option<String>,
    pub fuel_type: Option<String>,
    pub owner_name: String,
    pub owner_address: Option<String>,
    pub owner_national_id: Option<String>,
}

// ── Response DTOs ─────────────────────────────────────────────────────────────

/// Detailed view of a pending submission (for admin review)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingSubmissionDetail {
    pub id: Uuid,
    pub plate_number: String,
    pub agent_id: Uuid,
    pub agent_name: Option<String>,
    pub location: Option<SubmissionLocation>,
    /// URL or Base64 of front image
    pub front_image_url: Option<String>,
    /// URL or Base64 of back image
    pub back_image_url: Option<String>,
    pub notes: Option<String>,
    pub status: SubmissionStatus,
    pub submitted_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub rejection_reason: Option<String>,
    pub vehicle_data: Option<VehicleDataEntry>,
}

/// Lightweight list item for admin dashboard
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingSubmissionListItem {
    /// `None` for an S3 `unregistered/` entry — no row in `pending_submissions`.
    pub id: Option<Uuid>,
    pub plate_number: String,
    pub agent_name: Option<String>,
    pub status: SubmissionStatus,
    pub submitted_at: String,
    pub source: SubmissionSource,
}

/// Response after a review action
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSubmissionResponse {
    pub message: String,
    pub submission_id: Uuid,
    pub status: SubmissionStatus,
    /// Populated only on approval: the vehicle ID in the main DB
    pub vehicle_id: Option<Uuid>,
}

/// Response after creating a pending submission
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DataEntryResponse {
    pub message: String,
    pub submission_id: Uuid,
    /// Plate number that was submitted
    pub plate_number: String,
}

/// Audit log entry for a submission action
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionAuditLogEntry {
    pub id: Uuid,
    pub action: String,
    pub performed_by: Uuid,
    pub performer_name: Option<String>,
    pub reason: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: String,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    Approved,
    Rejected,
}

impl SubmissionStatus {
    /// Parse from a DB string value
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            _ => Self::Pending,
        }
    }

    /// Convert to the string stored in the DB
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

/// Query parameters for listing submissions with optional status filter
#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmissionListQuery {
    pub status: Option<String>,
}

/// Origin of a [`PendingSubmissionListItem`] entry.
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionSource {
    Submission,
    S3Unregistered,
}
