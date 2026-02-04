use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub member_id: Uuid,
    pub username: String,
    // Add other fields
}
