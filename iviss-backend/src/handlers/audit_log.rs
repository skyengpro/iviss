use crate::app_state::AppState;
use crate::dto::audit::{AuditLogListResponse, AuditLogQuery};
use crate::errors::AppError;
use crate::queries::audit_log_queries;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// List audit logs with pagination and filtering (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs",
    params(AuditLogQuery),
    responses(
        (status = 200, description = "Audit logs retrieved successfully", body = AuditLogListResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "listAuditLogs",
    security(("bearer_auth" = []))
)]
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (items, total) = audit_log_queries::list_audit_logs(&state.db, &query).await?;

    Ok((
        StatusCode::OK,
        Json(AuditLogListResponse {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
        }),
    ))
}

/// Export audit logs as CSV (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs/export",
    params(AuditLogQuery),
    responses(
        (status = 200, description = "CSV file", content_type = "text/csv"),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "exportAuditLogs",
    security(("bearer_auth" = []))
)]
pub async fn export_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let items = audit_log_queries::export_audit_logs(&state.db, &query).await?;

    // Build CSV
    let mut csv = String::from("ID,Timestamp,User ID,User Name,Action,Resource Type,Resource ID,IP Address,Before Snapshot,After Snapshot\n");

    for entry in &items {
        let before = entry
            .before_snapshot
            .as_ref()
            .map(|v: &serde_json::Value| v.to_string())
            .unwrap_or_default()
            .replace('"', "\"\"");
        let after = entry
            .after_snapshot
            .as_ref()
            .map(|v: &serde_json::Value| v.to_string())
            .unwrap_or_default()
            .replace('"', "\"\"");

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},\"{}\",\"{}\"\n",
            entry.id,
            entry.created_at,
            entry.user_id.map(|u: uuid::Uuid| u.to_string()).unwrap_or_default(),
            entry.user_name.as_deref().unwrap_or(""),
            entry.action.as_str(),
            entry.resource_type.as_deref().unwrap_or(""),
            entry.resource_id.map(|u: uuid::Uuid| u.to_string()).unwrap_or_default(),
            entry.ip_address.as_deref().unwrap_or(""),
            before,
            after,
        ));
    }

    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"audit_logs.csv\"",
        ),
    ];

    Ok((StatusCode::OK, headers, csv))
}
