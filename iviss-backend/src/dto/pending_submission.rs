use crate::dto::common::SubmissionLocation;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
// ── Request DTOs (admin actions) ──────────────────────────────────────────────

// ReviewSubmissionRequest and VehicleDataEntry removed as they are not yet used by any handler.
// Re-add when implementing admin review endpoints.

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
