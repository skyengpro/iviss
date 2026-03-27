use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// Tracks gray-card image submissions awaiting admin validation.
/// Maps to the `pending_submissions` table.
#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct PendingSubmission {
    pub id: Uuid,
    pub plate_number: String,
    pub agent_id: Uuid,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    /// URL to front image of carte grise
    pub front_image_url: Option<String>,
    /// URL to back image of carte grise
    pub back_image_url: Option<String>,
    pub notes: Option<String>,
    /// "pending" | "approved" | "rejected"
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<OffsetDateTime>,
    pub rejection_reason: Option<String>,
    /// Admin-entered vehicle details on approval (serialised JSON)
    pub vehicle_data: Option<serde_json::Value>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Row returned when joining the audit log table.
#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct SubmissionAuditLogRow {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub action: String,
    pub performed_by: Uuid,
    pub performer_name: Option<String>,
    pub reason: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: OffsetDateTime,
}
