use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Member {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    // Add other fields
}
