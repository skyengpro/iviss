use crate::dto::stats::{ActivityData, AgentLocationDto, DashboardStats};
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
        sqlx::query(
            "SELECT COUNT(*) FROM users WHERE role = 'agent' AND status = 'ACTIVE' AND deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?
        .get(0);

    // 5. Activity 24h (last 24 hours including current hour)
    let activity_24h: Vec<ActivityData> = sqlx::query_as(
        r#"
        WITH hours AS (
            SELECT generate_series(
                date_trunc('hour', NOW()) - interval '23 hours',
                date_trunc('hour', NOW()),
                interval '1 hour'
            ) AS hour
        )
        SELECT 
            to_char(h.hour, 'HH24:00') as hour,
            count(c.id) as count
        FROM hours h
        LEFT JOIN control_records c ON date_trunc('hour', c.created_at) = h.hour
        GROUP BY h.hour
        ORDER BY h.hour
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    // 6. Live agents
    let live_agents: Vec<AgentLocationDto> = sqlx::query_as(
        r#"
        SELECT 
            al.agent_id,
            u.full_name as agent_name,
            al.latitude,
            al.longitude,
            to_char(al.updated_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') as last_updated
        FROM agent_locations al
        JOIN users u ON al.agent_id = u.id
        WHERE al.updated_at >= NOW() - interval '30 minutes'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(DashboardStats {
        today_controls,
        active_alerts,
        total_vehicles,
        online_agents,
        activity_24h,
        live_agents,
    })
}
