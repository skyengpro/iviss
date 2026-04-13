use crate::dto::audit::{AuditAction, AuditLogEntry, AuditLogQuery};
use crate::errors::AppError;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Parameters for inserting a new audit log entry.
/// Using a struct avoids the clippy `too_many_arguments` lint.
pub struct InsertAuditLogParams {
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub ip_address: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub before_snapshot: Option<serde_json::Value>,
    pub after_snapshot: Option<serde_json::Value>,
}

/// Insert a new audit log entry.
pub async fn insert_audit_log(
    pool: &PgPool,
    params: InsertAuditLogParams,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            user_id, action, ip_address, resource_type, resource_id,
            metadata, before_snapshot, after_snapshot
        )
        VALUES (
            $1, $2, $3::inet, $4, $5, $6, $7, $8
        )
        "#,
    )
    .bind(params.user_id)
    .bind(params.action)
    .bind(params.ip_address.as_deref())
    .bind(params.resource_type.as_deref())
    .bind(params.resource_id)
    .bind(params.metadata.unwrap_or(serde_json::json!({})))
    .bind(params.before_snapshot)
    .bind(params.after_snapshot)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert audit log: {:?}", e);
        AppError::database(e)
    })?;

    Ok(())
}

/// List audit logs with pagination and filtering.
pub async fn list_audit_logs(
    pool: &PgPool,
    query: &AuditLogQuery,
) -> Result<(Vec<AuditLogEntry>, i64), AppError> {
    let offset = (query.page - 1).max(0) * query.page_size;

    let mut count_builder: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT COUNT(*) AS total FROM audit_logs al WHERE 1=1");

    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        r#"SELECT
            al.id,
            al.user_id,
            u.full_name AS user_name,
            al.action,
            al.resource_type,
            al.resource_id,
            al.ip_address::text AS ip_address,
            al.metadata,
            al.before_snapshot,
            al.after_snapshot,
            TO_CHAR(al.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at_iso
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        WHERE 1=1"#,
    );

    apply_filters(&mut query_builder, &mut count_builder, query);

    // Get total count
    let count_row = count_builder
        .build()
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;
    let total: i64 = count_row.get("total");

    // Get paginated results
    query_builder.push(" ORDER BY al.created_at DESC");
    query_builder.push(format!(" LIMIT {} OFFSET {}", query.page_size, offset));

    let rows = query_builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let items = map_rows(rows);
    Ok((items, total))
}

/// Export all audit logs matching the filter (no pagination limit, max 10 000 rows).
pub async fn export_audit_logs(
    pool: &PgPool,
    query: &AuditLogQuery,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        r#"SELECT
            al.id,
            al.user_id,
            u.full_name AS user_name,
            al.action,
            al.resource_type,
            al.resource_id,
            al.ip_address::text AS ip_address,
            al.metadata,
            al.before_snapshot,
            al.after_snapshot,
            TO_CHAR(al.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at_iso
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        WHERE 1=1"#,
    );

    // Use a dummy count builder so we can reuse apply_filters signature
    let dummy_count: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT 1");
    apply_export_filters(&mut query_builder, query);
    let _ = dummy_count; // silence unused warning

    query_builder.push(" ORDER BY al.created_at DESC LIMIT 10000");

    let rows = query_builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(map_rows(rows))
}

/// Get all audit logs belonging to a specific user (most recent first).
pub async fn get_logs_by_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let rows = sqlx::query(
        r#"SELECT
            al.id, al.user_id, u.full_name AS user_name, al.action,
            al.resource_type, al.resource_id,
            al.ip_address::text AS ip_address,
            al.metadata, al.before_snapshot, al.after_snapshot,
            TO_CHAR(al.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at_iso
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        WHERE al.user_id = $1
        ORDER BY al.created_at DESC
        LIMIT $2"#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(map_rows(rows))
}

/// Get all audit logs for a specific resource (e.g. vehicle, pending_submission).
pub async fn get_logs_by_resource(
    pool: &PgPool,
    resource_type: &str,
    resource_id: Uuid,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let rows = sqlx::query(
        r#"SELECT
            al.id, al.user_id, u.full_name AS user_name, al.action,
            al.resource_type, al.resource_id,
            al.ip_address::text AS ip_address,
            al.metadata, al.before_snapshot, al.after_snapshot,
            TO_CHAR(al.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at_iso
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        WHERE al.resource_type = $1 AND al.resource_id = $2
        ORDER BY al.created_at DESC"#,
    )
    .bind(resource_type)
    .bind(resource_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(map_rows(rows))
}

/// Get the N most recent audit log entries (for dashboard / live feed).
pub async fn get_recent_audit_logs(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let rows = sqlx::query(
        r#"SELECT
            al.id, al.user_id, u.full_name AS user_name, al.action,
            al.resource_type, al.resource_id,
            al.ip_address::text AS ip_address,
            al.metadata, al.before_snapshot, al.after_snapshot,
            TO_CHAR(al.created_at, 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at_iso
        FROM audit_logs al
        LEFT JOIN users u ON al.user_id = u.id
        ORDER BY al.created_at DESC
        LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(map_rows(rows))
}

/// Count audit log entries for a specific action within the last `hours` hours.
pub async fn count_by_action_since(
    pool: &PgPool,
    action: AuditAction,
    hours: i64,
) -> Result<i64, AppError> {
    let row = sqlx::query(
        "SELECT COUNT(*) AS total FROM audit_logs WHERE action = $1 AND created_at >= NOW() - ($2 || ' hours')::interval",
    )
    .bind(action)
    .bind(hours)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(row.get("total"))
}

// ─── internal helpers ────────────────────────────────────────────────────────

fn apply_filters(
    q: &mut sqlx::QueryBuilder<sqlx::Postgres>,
    c: &mut sqlx::QueryBuilder<sqlx::Postgres>,
    query: &AuditLogQuery,
) {
    if let Some(ref start_date) = query.start_date {
        let clause = format!(" AND al.created_at >= '{} 00:00:00'::timestamp", start_date);
        q.push(&clause);
        c.push(&clause);
    }
    if let Some(ref end_date) = query.end_date {
        let clause = format!(" AND al.created_at <= '{} 23:59:59'::timestamp", end_date);
        q.push(&clause);
        c.push(&clause);
    }
    if let Some(ref user_id) = query.user_id {
        let clause = format!(" AND al.user_id = '{}'", user_id);
        q.push(&clause);
        c.push(&clause);
    }
    if let Some(ref action) = query.action {
        let clause = format!(" AND al.action = '{}'::audit_action", action);
        q.push(&clause);
        c.push(&clause);
    }
    if let Some(ref resource_type) = query.resource_type {
        let clause = format!(" AND al.resource_type = '{}'", resource_type);
        q.push(&clause);
        c.push(&clause);
    }
}

fn apply_export_filters(q: &mut sqlx::QueryBuilder<sqlx::Postgres>, query: &AuditLogQuery) {
    if let Some(ref start_date) = query.start_date {
        q.push(format!(
            " AND al.created_at >= '{} 00:00:00'::timestamp",
            start_date
        ));
    }
    if let Some(ref end_date) = query.end_date {
        q.push(format!(
            " AND al.created_at <= '{} 23:59:59'::timestamp",
            end_date
        ));
    }
    if let Some(ref user_id) = query.user_id {
        q.push(format!(" AND al.user_id = '{}'", user_id));
    }
    if let Some(ref action) = query.action {
        q.push(format!(" AND al.action = '{}'::audit_action", action));
    }
    if let Some(ref resource_type) = query.resource_type {
        q.push(format!(" AND al.resource_type = '{}'", resource_type));
    }
}

fn map_rows(rows: Vec<sqlx::postgres::PgRow>) -> Vec<AuditLogEntry> {
    rows.into_iter()
        .map(|row| AuditLogEntry {
            id: row.get("id"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            action: row.get("action"),
            resource_type: row.get("resource_type"),
            resource_id: row.get("resource_id"),
            ip_address: row.get("ip_address"),
            metadata: row.get("metadata"),
            before_snapshot: row.get("before_snapshot"),
            after_snapshot: row.get("after_snapshot"),
            created_at: row.get("created_at_iso"),
        })
        .collect()
}
