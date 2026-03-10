use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityData {
    pub hour: String,
    pub count: i64,
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
    pub activity_24h: Vec<ActivityData>,
    pub live_agents: Vec<AgentLocationDto>,
}
