use crate::dto::common::{IdentificationMode, Status};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Request

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct VehicleSearchRequest {
    pub plate: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub agent_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
}

// Sub-objects

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct OwnerInfo {
    pub name: Option<String>,
    pub address: Option<String>,
    pub national_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct VehicleInfo {
    pub brand: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub engine_power: Option<String>,
    pub fuel_type: Option<String>,
    pub chassis_number: Option<String>,
    pub customs_status: Option<String>,
    pub owner: OwnerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct InsuranceStatus {
    pub status: Status,
    pub provider: Option<String>,
    pub policy_number: Option<String>,
    pub expiry_date: Option<String>,
    pub coverage_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct PoliceStatus {
    pub status: Status,
    pub is_wanted: bool,
    pub is_stolen: bool,
    pub report_date: Option<String>,
    pub report_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct CustomsStatus {
    pub status: Status,
    pub is_cleared: bool,
    pub import_date: Option<String>,
    pub declaration_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct TechnicalStatus {
    pub status: Status,
    pub last_inspection_date: Option<String>,
    pub expiry_date: Option<String>,
    pub mileage: Option<i64>,
    pub defects: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct StatusResults {
    pub overall_status: Status,
    pub insurance: InsuranceStatus,
    pub police: PoliceStatus,
    pub customs: CustomsStatus,
    pub technical: TechnicalStatus,
    pub vehicle_image_url: Option<String>,
}

//  Response

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum VehicleDataSource {
    Live,
    Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct VehicleSearchResult {
    pub plate_number: String,
    pub confidence: Option<f64>,
    pub identification_mode: Option<IdentificationMode>,
    pub vehicle: VehicleInfo,
    pub status_results: StatusResults,
    pub source: Option<VehicleDataSource>,
    pub cached_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct UploadResponse {
    pub message: String,
    pub submission_id: String,
}
