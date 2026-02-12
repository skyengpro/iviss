use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// Tracks gray-card image submissions awaiting super-admin validation
/// Maps to a `pending_submissions` table (to be created in migration)
#[derive(Debug, FromRow)]
pub struct PendingSubmission {
    pub id: Uuid,
    pub plate_number: String,
    pub agent_id: Uuid,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    /// Path to front image file in storage
    pub front_image_path: String,
    /// Path to back image file in storage
    pub back_image_path: String,
    pub notes: Option<String>,
    pub status: String,              // "pending" | "approved" | "rejected"
    pub created_at: OffsetDateTime,
}