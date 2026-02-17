use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::dto::common::IdentificationMode;
use crate::dto::list_control::ControlResults;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateControlRequest {
    pub plate_number: String,
    pub agent_id: Uuid,
    pub organization_id: Uuid,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub identification_mode: IdentificationMode,
    pub ocr_confidence: Option<f64>,
    pub results: ControlResults,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateControlResponse {
    pub id: Uuid,
    pub message: String,
}
