use crate::dto::common::SubmissionLocation;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
// ── Request DTOs (admin actions) ──────────────────────────────────────────────

/// Admin decision on a pending submission
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewSubmissionRequest {
    /// "approved" | "rejected"
    pub decision: SubmissionDecision,
    /// Optional note from admin explaining the decision
    pub admin_note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionDecision {
    Approved,
    Rejected,
}

/// Admin data entry after approving a gray card submission
#[derive(Debug, Deserialize, ToSchema)]
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
    pub carte_grise_expiry: Option<String>,
}

/// Detailed view of a pending submission (for admin review)
#[derive(Debug, Serialize, ToSchema)]
pub struct PendingSubmissionRequest {
    pub id: Uuid,
    pub plate_number: String,
    pub agent_id: Uuid,
    pub agent_name: Option<String>,
    pub location: Option<SubmissionLocation>,
    /// URL or Base64 of front image
    pub front_image_url: String,
    /// URL or Base64 of back image
    pub back_image_url: String,
    pub notes: Option<String>,
    pub status: SubmissionStatus,
    pub submitted_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub admin_note: Option<String>,
}

/// Lightweight list item for admin dashboard
#[derive(Debug, Serialize, ToSchema)]
pub struct PendingSubmissionListItem {
    pub id: Uuid,
    pub plate_number: String,
    pub agent_name: Option<String>,
    pub status: SubmissionStatus,
    pub submitted_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    Pending,
    Approved,
    Rejected,
}

/// Response after approved and enters vehicle data
#[derive(Debug, Serialize, ToSchema)]
pub struct DataEntryResponse {
    pub message: String,
    pub submission_id: Uuid,
    /// Plate number that was added to the external registry
    pub plate_number: String,
}
