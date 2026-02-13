use crate::dto::stats::DashboardStats;
use crate::errors::AppError;
use sqlx::{PgPool, Row};

pub async fn get_dashboard_stats_query(pool: &PgPool) -> Result<DashboardStats, AppError> {
    // 1. Today controls
    let today_controls: i64 =
        sqlx::query("SELECT COUNT(*) FROM control_records WHERE created_at >= CURRENT_DATE")
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?
            .get(0);

    // 2. Active alerts (critical status controls today? or just critical status in general? assuming critical status vehicle checks today)
    let active_alerts: i64 = sqlx::query(
        "SELECT COUNT(*) FROM control_records WHERE overall_status = 'critical' AND created_at >= CURRENT_DATE"
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?
    .get(0);

    // 3. Total vehicles
    let total_vehicles: i64 = sqlx::query("SELECT COUNT(*) FROM vehicles WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?
        .get(0);

    // 4. Online agents (Approximation: agents with controls in last hour?? Or just active agents count?
    // Requirement says 'onlineAgents', usually implies realtime.
    // For now, let's count agents who performed a check in the last 24h as 'active recently' or just count total active agents)
    // Let's go with "Active Agents" in the system for now as we don't have heartbeat.
    let online_agents: i64 =
        sqlx::query("SELECT COUNT(*) FROM users WHERE role = 'agent' AND is_active = true")
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?
            .get(0);

    Ok(DashboardStats {
        today_controls,
        active_alerts,
        total_vehicles,
        online_agents,
    })
}
