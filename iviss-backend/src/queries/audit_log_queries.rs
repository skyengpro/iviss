use crate::dto::audit::{AuditAction, AuditLogEntry, AuditLogQuery};
use crate::errors::AppError;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Insert a new audit log entry.
pub async fn insert_audit_log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: AuditAction,
    ip_address: Option<&str>,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
    before_snapshot: Option<serde_json::Value>,
    after_snapshot: Option<serde_json::Value>,
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
    .bind(user_id)
    .bind(action)
    .bind(ip_address)
    .bind(resource_type)
    .bind(resource_id)
    .bind(metadata.unwrap_or(serde_json::json!({})))
    .bind(before_snapshot)
    .bind(after_snapshot)
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

    // Build WHERE clauses dynamically using QueryBuilder
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

    if let Some(ref start_date) = query.start_date {
        let clause = format!(" AND al.created_at >= '{} 00:00:00'::timestamp", start_date);
        query_builder.push(&clause);
        count_builder.push(&clause);
    }

    if let Some(ref end_date) = query.end_date {
        let clause = format!(" AND al.created_at <= '{} 23:59:59'::timestamp", end_date);
        query_builder.push(&clause);
        count_builder.push(&clause);
    }

    if let Some(ref user_id) = query.user_id {
        let clause = format!(" AND al.user_id = '{}'", user_id);
        query_builder.push(&clause);
        count_builder.push(&clause);
    }

    if let Some(ref action) = query.action {
        let clause = format!(" AND al.action = '{}'::audit_action", action);
        query_builder.push(&clause);
        count_builder.push(&clause);
    }

    if let Some(ref resource_type) = query.resource_type {
        let clause = format!(" AND al.resource_type = '{}'", resource_type);
        query_builder.push(&clause);
        count_builder.push(&clause);
    }

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

    let items: Vec<AuditLogEntry> = rows
        .into_iter()
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
        .collect();

    Ok((items, total))
}

/// Export all audit logs matching the filter (no pagination limit).
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

    if let Some(ref start_date) = query.start_date {
        query_builder.push(format!(
            " AND al.created_at >= '{} 00:00:00'::timestamp",
            start_date
        ));
    }

    if let Some(ref end_date) = query.end_date {
        query_builder.push(format!(
            " AND al.created_at <= '{} 23:59:59'::timestamp",
            end_date
        ));
    }

    if let Some(ref user_id) = query.user_id {
        query_builder.push(format!(" AND al.user_id = '{}'", user_id));
    }

    if let Some(ref action) = query.action {
        query_builder.push(format!(" AND al.action = '{}'::audit_action", action));
    }

    if let Some(ref resource_type) = query.resource_type {
        query_builder.push(format!(" AND al.resource_type = '{}'", resource_type));
    }

    query_builder.push(" ORDER BY al.created_at DESC LIMIT 10000");

    let rows = query_builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let items: Vec<AuditLogEntry> = rows
        .into_iter()
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
        .collect();

    Ok(items)
}
