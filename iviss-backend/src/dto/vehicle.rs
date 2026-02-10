use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct OwnerInfo {
    pub full_name: String,
    pub cni_number: String,
    pub phone_number: Option<String>,
}

/// Aggregated status from partner APIs
#[derive(Debug, Serialize, ToSchema)]
pub struct VehicleStatus {
    pub insurance_valid: bool,
    pub customs_cleared: bool,
    pub inspection_valid: bool,
    pub is_wanted: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VehicleResponse {
    pub plate: String,
    pub chassis: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub power: Option<String>,
    pub carte_grise_expiry: Option<String>,
    pub owner: OwnerInfo,
    /// Real-time status from partner APIs
    pub status: VehicleStatus,
}

/// Returned after a successful gray-card image upload
#[derive(Debug, Serialize, ToSchema)]
pub struct UploadResponse {
    pub message: String,
    /// Reference ID to track the uploaded file
    pub file_id: String,
}

/// Query parameter for plate lookup
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlateQuery {
    pub plate_number: String,
}