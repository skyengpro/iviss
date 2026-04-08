use crate::dto::audit::{AuditLogEntry, AuditLogQuery, AuditAction};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn get_audit_logs(
    pool: &PgPool,
    query: AuditLogQuery,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    let mut sql = r#"
        SELECT a.id, a.user_id, a.device_id, a.action, a.metadata, a.created_at,
               u.full_name as user_name
        FROM audit_logs a
        LEFT JOIN users u ON a.user_id = u.id
        WHERE 1=1
    "#.to_string();

    if query.user_id.is_some() {
        sql.push_str(" AND a.user_id = $3");
    }
    if query.action.is_some() {
        sql.push_str(" AND a.action = $4::audit_action");
    }

    sql.push_str(" ORDER BY a.created_at DESC LIMIT $1 OFFSET $2");

    let mut sql_query = sqlx::query(&sql)
        .bind(limit)
        .bind(offset);

    if let Some(user_id) = query.user_id {
        sql_query = sql_query.bind(user_id);
    }
    if let Some(action) = query.action {
        sql_query = sql_query.bind(action);
    }

    let rows = sql_query.fetch_all(pool).await.map_err(AppError::database)?;

    let mut entries = Vec::new();
    for row in rows {
        let action_str: String = row.try_get("action").map_err(AppError::database)?;
        let created_at: time::OffsetDateTime = row.try_get("created_at").map_err(AppError::database)?;

        entries.push(AuditLogEntry {
            id: row.try_get("id").map_err(AppError::database)?,
            user_id: row.try_get("user_id").ok(),
            user_name: row.try_get("user_name").ok(),
            device_id: row.try_get("device_id").ok(),
            action: AuditAction::from_str(&action_str).unwrap_or(AuditAction::LoginSuccess), // Fallback
            metadata: row.try_get("metadata").map_err(AppError::database)?,
            created_at: created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        });
    }

    Ok(entries)
}

pub async fn export_audit_logs_csv(
    pool: &PgPool,
) -> Result<String, AppError> {
    // For export, we fetch a larger chunk (e.g., last 1000 logs)
    let rows = sqlx::query(
        r#"
        SELECT a.id, a.user_id, a.action, a.created_at, u.full_name as user_name
        FROM audit_logs a
        LEFT JOIN users u ON a.user_id = u.id
        ORDER BY a.created_at DESC
        LIMIT 1000
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut csv = String::from("ID,Timestamp,User,Action\n");
    for row in rows {
        let id: Uuid = row.get("id");
        let timestamp: time::OffsetDateTime = row.get("created_at");
        let user: Option<String> = row.get("user_name");
        let action: String = row.get("action");

        csv.push_str(&format!(
            "{},{},{},{}\n",
            id,
            timestamp.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
            user.unwrap_or_else(|| "System".to_string()),
            action
        ));
    }

    Ok(csv)
}
