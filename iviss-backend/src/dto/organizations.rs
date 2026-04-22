use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationType {
    Police,
    Customs,
    BorderControl,
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub org_type: OrganizationType,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub org_type: OrganizationType,
    pub region: Option<String>,
    pub start_work_time: Option<u32>,
    pub end_work_time: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub org_type: Option<OrganizationType>,
    pub region: Option<String>,
    pub start_work_time: Option<u32>,
    pub end_work_time: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationDetails {
    pub id: Uuid,
    pub name: String,
    pub org_type: OrganizationType,
    pub region: Option<String>,
    pub user_count: i64,
    pub active_agents: i64,
    pub control_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub shift_start_hour: u32,
    pub shift_end_hour: u32,
}
