use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")] // ← converts snake_case → camelCase in JSON
pub struct DashboardStats {
    pub today_controls: i64,
    pub active_alerts: i64,
    pub total_vehicles: i64,
    pub online_agents: i64,
}
