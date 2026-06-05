use sqlx::FromRow;

/// Maps to the external PostgreSQL 9.4 national vehicle registry
/// Field names match the external DB columns — do NOT rename
#[derive(Debug, FromRow)]
pub struct VehicleRow {
    pub plate_number: String,
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
