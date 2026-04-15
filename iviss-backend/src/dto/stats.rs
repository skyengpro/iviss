use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityData {
    pub hour: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DashboardRange {
    #[serde(rename = "24h")]
    H24,
    #[serde(rename = "7d")]
    D7,
    #[serde(rename = "30d")]
    D30,
}

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ControlActivityPoint {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ControlActivityResponse {
    pub range: DashboardRange,
    pub series: Vec<ControlActivityPoint>,
}

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TopAgentDto {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub organization_name: String,
    pub controls_count: i64,
    pub is_online: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopAgentsResponse {
    pub range: DashboardRange,
    pub agents: Vec<TopAgentDto>,
}

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFeedItemDto {
    pub id: Uuid,
    pub plate_number: String,
    pub overall_status: String,
    pub created_at: String,
    pub agent_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFeedResponse {
    pub items: Vec<ActivityFeedItemDto>,
}

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentAlertItemDto {
    pub id: Uuid,
    pub plate_number: String,
    pub overall_status: String,
    pub created_at: String,
    pub agent_name: String,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecentAlertsResponse {
    pub items: Vec<RecentAlertItemDto>,
}

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentLocationDto {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub last_updated: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")] // ← converts snake_case → camelCase in JSON
pub struct DashboardStats {
    pub today_controls: i64,
    pub active_alerts: i64,
    pub total_vehicles: i64,
    pub online_agents: i64,
    pub pending_submissions: i64,
    pub organizations_count: i64,
    pub activity_24h: Vec<ActivityData>,
    pub live_agents: Vec<AgentLocationDto>,
}

/// Org-scoped dashboard stats — returned for org_admin users.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrgDashboardStats {
    pub organization_name: String,
    pub today_controls: i64,
    pub active_alerts: i64,
    pub online_agents: i64,
    pub pending_submissions: i64,
    pub total_users: i64,
    pub activity_24h: Vec<ActivityData>,
    pub live_agents: Vec<AgentLocationDto>,
}
