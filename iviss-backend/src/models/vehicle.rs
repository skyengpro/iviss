use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vehicle {
    pub plate_number: String,
    pub chassis_number: String,
    // Add other fields
}
