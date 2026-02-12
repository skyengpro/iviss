use crate::dto::common::{IdentificationMode, Status};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GpsPosition {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListControlRequest {
    pub plate_number: String,
    pub agent_id: Uuid,
    pub identification_mode: IdentificationMode,
    pub position: GpsPosition,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ControlListQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub agent_id: Option<Uuid>,
    pub status: Option<Status>,
    pub plate: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ControlLocation {
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ControlResults {
    pub registration: Status,
    pub insurance: Status,
    pub technical_inspection: Status,
    pub wanted_status: Status,
    pub customs_status: Status,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Check,
    Flag,
    Citation,
    Impound,
    Release,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ControlAction {
    pub action_type: ActionType,
    pub description: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListControlResponse {
    pub id: Uuid,
    pub plate_number: String,
    pub agent_name: Option<String>,
    pub agent_id: Uuid,
    pub organization_id: Uuid,
    pub timestamp: String,
    pub status: Status,
    pub identification_mode: IdentificationMode,
    pub confidence: Option<f64>,
    pub location: ControlLocation,
    pub results: ControlResults,
    pub actions: Vec<ControlAction>,
    pub notes: Option<String>,
}
