use crate::dto::stats::{
    ActivityData, ActivityFeedItemDto, AgentLocationDto, ControlActivityPoint, DashboardRange,
    DashboardStats, OrgDashboardStats, RecentAlertItemDto, TopAgentDto,
};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

fn range_to_sql_window(range: DashboardRange) -> &'static str {
    match range {
        DashboardRange::H24 => "24 hours",
        DashboardRange::D7 => "7 days",
        DashboardRange::D30 => "30 days",
    }
}

pub async fn get_control_activity_series_query(
    pool: &PgPool,
    range: DashboardRange,
) -> Result<Vec<ControlActivityPoint>, AppError> {
    match range {
        DashboardRange::H24 => {
            let series: Vec<ControlActivityPoint> = sqlx::query_as(
                r#"
                WITH hours AS (
                    SELECT generate_series(
                        date_trunc('hour', NOW()) - interval '23 hours',
                        date_trunc('hour', NOW()),
                        interval '1 hour'
                    ) AS bucket
                )
                SELECT 
                    to_char(h.bucket, 'HH24:00') as label,
                    count(c.id) as count
                FROM hours h
                LEFT JOIN control_records c ON date_trunc('hour', c.created_at) = h.bucket
                GROUP BY h.bucket
                ORDER BY h.bucket
                "#,
            )
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?;
            Ok(series)
        }
        DashboardRange::D7 | DashboardRange::D30 => {
            let window = range_to_sql_window(range);
            let query = format!(
                r#"
                WITH days AS (
                    SELECT generate_series(
                        date_trunc('day', NOW()) - interval '{window}' + interval '1 day',
                        date_trunc('day', NOW()),
                        interval '1 day'
                    ) AS bucket
                )
                SELECT 
                    to_char(d.bucket, 'DD Mon') as label,
                    count(c.id) as count
                FROM days d
                LEFT JOIN control_records c ON date_trunc('day', c.created_at) = d.bucket
                GROUP BY d.bucket
                ORDER BY d.bucket
                "#
            );

            let series: Vec<ControlActivityPoint> = sqlx::query_as(&query)
                .fetch_all(pool)
                .await
                .map_err(AppError::database)?;
            Ok(series)
        }
    }
}

pub async fn get_top_agents_query(
    pool: &PgPool,
    range: DashboardRange,
    limit: i64,
) -> Result<Vec<TopAgentDto>, AppError> {
    let window = range_to_sql_window(range);
    let query = format!(
        r#"
        SELECT
            u.id as agent_id,
            u.full_name as agent_name,
            o.name as organization_name,
            COUNT(c.id) as controls_count,
            (MAX(al.updated_at) IS NOT NULL AND MAX(al.updated_at) >= NOW() - interval '30 minutes') as is_online
        FROM users u
        JOIN organizations o ON o.id = u.organization_id
        LEFT JOIN control_records c
            ON c.agent_id = u.id
           AND c.created_at >= NOW() - interval '{window}'
        LEFT JOIN agent_locations al ON al.agent_id = u.id
        WHERE u.role = 'agent'
          AND u.deleted_at IS NULL
        GROUP BY u.id, u.full_name, o.name
        ORDER BY controls_count DESC, agent_name ASC
        LIMIT $1
        "#
    );

    let agents: Vec<TopAgentDto> = sqlx::query_as(&query)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(agents)
}

pub async fn get_activity_feed_query(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ActivityFeedItemDto>, AppError> {
    let items: Vec<ActivityFeedItemDto> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.plate_number,
            c.overall_status,
            to_char(c.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') as created_at,
            u.full_name as agent_name
        FROM control_records c
        JOIN users u ON u.id = c.agent_id
        WHERE c.deleted_at IS NULL
        ORDER BY c.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(items)
}

pub async fn get_recent_alerts_query(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentAlertItemDto>, AppError> {
    let items: Vec<RecentAlertItemDto> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.plate_number,
            c.overall_status,
            to_char(c.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') as created_at,
            u.full_name as agent_name,
            c.address
        FROM control_records c
        JOIN users u ON u.id = c.agent_id
        WHERE c.deleted_at IS NULL
          AND c.overall_status IN ('warning', 'critical')
        ORDER BY c.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(items)
}

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

    // 5. Pending submissions (back-office workload)
    let pending_submissions: i64 =
        sqlx::query("SELECT COUNT(*) FROM pending_submissions WHERE status = 'pending'")
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?
            .get(0);

    // 6. Organizations count
    let organizations_count: i64 =
        sqlx::query("SELECT COUNT(*) FROM organizations WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?
            .get(0);

    // 7. Activity 24h (last 24 hours including current hour)
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

    // 8. Live agents
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
        pending_submissions,
        organizations_count,
        activity_24h,
        live_agents,
    })
}

// ══════════════════════════════════════════════════════════════════════════════
// Org-scoped queries — filter everything by organization_id
// ══════════════════════════════════════════════════════════════════════════════

pub async fn get_org_dashboard_stats_query(
    pool: &PgPool,
    org_id: Uuid,
) -> Result<OrgDashboardStats, AppError> {
    // 0. Organization name
    let org_name: String =
        sqlx::query("SELECT name FROM organizations WHERE id = $1 AND deleted_at IS NULL")
            .bind(org_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?
            .get(0);

    // 1. Today controls (org-scoped)
    let today_controls: i64 = sqlx::query(
        "SELECT COUNT(*) FROM control_records WHERE organization_id = $1 AND created_at >= CURRENT_DATE",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?
    .get(0);

    // 2. Active alerts (org-scoped)
    let active_alerts: i64 = sqlx::query(
        "SELECT COUNT(*) FROM control_records WHERE organization_id = $1 AND overall_status = 'critical' AND created_at >= CURRENT_DATE",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?
    .get(0);

    // 3. Online agents (org-scoped)
    let online_agents: i64 = sqlx::query(
        "SELECT COUNT(*) FROM users WHERE organization_id = $1 AND role = 'agent' AND status = 'ACTIVE' AND deleted_at IS NULL",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?
    .get(0);

    // 4. Pending submissions (org-scoped via agent's org)
    let pending_submissions: i64 = sqlx::query(
        r#"
        SELECT COUNT(*)
        FROM pending_submissions ps
        JOIN users u ON u.id = ps.agent_id
        WHERE u.organization_id = $1 AND ps.status = 'pending'
        "#,
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?
    .get(0);

    // 5. Total users in this org
    let total_users: i64 =
        sqlx::query("SELECT COUNT(*) FROM users WHERE organization_id = $1 AND deleted_at IS NULL")
            .bind(org_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?
            .get(0);

    // 6. Activity 24h (org-scoped)
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
        LEFT JOIN control_records c
            ON date_trunc('hour', c.created_at) = h.hour
           AND c.organization_id = $1
        GROUP BY h.hour
        ORDER BY h.hour
        "#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    // 7. Live agents (org-scoped)
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
        WHERE u.organization_id = $1
          AND al.updated_at >= NOW() - interval '30 minutes'
        "#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(OrgDashboardStats {
        organization_name: org_name,
        today_controls,
        active_alerts,
        online_agents,
        pending_submissions,
        total_users,
        activity_24h,
        live_agents,
    })
}

pub async fn get_org_activity_feed_query(
    pool: &PgPool,
    org_id: Uuid,
    limit: i64,
) -> Result<Vec<ActivityFeedItemDto>, AppError> {
    let items: Vec<ActivityFeedItemDto> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.plate_number,
            c.overall_status,
            to_char(c.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') as created_at,
            u.full_name as agent_name
        FROM control_records c
        JOIN users u ON u.id = c.agent_id
        WHERE c.organization_id = $1 AND c.deleted_at IS NULL
        ORDER BY c.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(org_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(items)
}

pub async fn get_org_recent_alerts_query(
    pool: &PgPool,
    org_id: Uuid,
    limit: i64,
) -> Result<Vec<RecentAlertItemDto>, AppError> {
    let items: Vec<RecentAlertItemDto> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.plate_number,
            c.overall_status,
            to_char(c.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') as created_at,
            u.full_name as agent_name,
            c.address
        FROM control_records c
        JOIN users u ON u.id = c.agent_id
        WHERE c.organization_id = $1
          AND c.deleted_at IS NULL
          AND c.overall_status IN ('warning', 'critical')
        ORDER BY c.created_at DESC
        LIMIT $2
        "#,
    )
    .bind(org_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(items)
}

pub async fn get_org_top_agents_query(
    pool: &PgPool,
    org_id: Uuid,
    range: DashboardRange,
    limit: i64,
) -> Result<Vec<TopAgentDto>, AppError> {
    let window = range_to_sql_window(range);
    let query = format!(
        r#"
        SELECT
            u.id as agent_id,
            u.full_name as agent_name,
            o.name as organization_name,
            COUNT(c.id) as controls_count,
            (MAX(al.updated_at) IS NOT NULL AND MAX(al.updated_at) >= NOW() - interval '30 minutes') as is_online
        FROM users u
        JOIN organizations o ON o.id = u.organization_id
        LEFT JOIN control_records c
            ON c.agent_id = u.id
           AND c.created_at >= NOW() - interval '{window}'
        LEFT JOIN agent_locations al ON al.agent_id = u.id
        WHERE u.organization_id = $1
          AND u.role = 'agent'
          AND u.deleted_at IS NULL
        GROUP BY u.id, u.full_name, o.name
        ORDER BY controls_count DESC, agent_name ASC
        LIMIT $2
        "#
    );

    let agents: Vec<TopAgentDto> = sqlx::query_as(&query)
        .bind(org_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(agents)
}

pub async fn get_org_control_activity_series_query(
    pool: &PgPool,
    org_id: Uuid,
    range: DashboardRange,
) -> Result<Vec<ControlActivityPoint>, AppError> {
    match range {
        DashboardRange::H24 => {
            let series: Vec<ControlActivityPoint> = sqlx::query_as(
                r#"
                WITH hours AS (
                    SELECT generate_series(
                        date_trunc('hour', NOW()) - interval '23 hours',
                        date_trunc('hour', NOW()),
                        interval '1 hour'
                    ) AS bucket
                )
                SELECT
                    to_char(h.bucket, 'HH24:00') as label,
                    count(c.id) as count
                FROM hours h
                LEFT JOIN control_records c
                    ON date_trunc('hour', c.created_at) = h.bucket
                   AND c.organization_id = $1
                GROUP BY h.bucket
                ORDER BY h.bucket
                "#,
            )
            .bind(org_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?;
            Ok(series)
        }
        DashboardRange::D7 | DashboardRange::D30 => {
            let window = range_to_sql_window(range);
            let query = format!(
                r#"
                WITH days AS (
                    SELECT generate_series(
                        date_trunc('day', NOW()) - interval '{window}' + interval '1 day',
                        date_trunc('day', NOW()),
                        interval '1 day'
                    ) AS bucket
                )
                SELECT
                    to_char(d.bucket, 'DD Mon') as label,
                    count(c.id) as count
                FROM days d
                LEFT JOIN control_records c
                    ON date_trunc('day', c.created_at) = d.bucket
                   AND c.organization_id = $1
                GROUP BY d.bucket
                ORDER BY d.bucket
                "#
            );

            let series: Vec<ControlActivityPoint> = sqlx::query_as(&query)
                .bind(org_id)
                .fetch_all(pool)
                .await
                .map_err(AppError::database)?;
            Ok(series)
        }
    }
}
