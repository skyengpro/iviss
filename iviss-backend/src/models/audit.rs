use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub action: String,
    pub created_at: DateTime<Utc>,
}
