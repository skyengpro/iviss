use crate::app_state::AppState;
#[allow(unused_imports)]
use crate::dto::audit::AuditLogEntry;
use crate::dto::audit::AuditLogQuery;
use crate::errors::AppError;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

/// List general system audit logs (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit",
    tag = "admin",
    operation_id = "listAuditLogs",
    params(AuditLogQuery),
    responses(
        (status = 200, description = "List of audit log entries", body = [AuditLogEntry]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let logs = crate::queries::audit_queries::get_audit_logs(&state.db, query).await?;
    Ok((StatusCode::OK, Json(logs)))
}

/// Export audit logs as CSV (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit/export",
    tag = "admin",
    operation_id = "exportAuditLogs",
    responses(
        (status = 200, description = "Audit logs in CSV format", body = String),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_audit_logs(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let csv_data = crate::queries::audit_queries::export_audit_logs_csv(&state.db).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"audit_logs.csv\"",
        )
        .body(csv_data)
        .map_err(|e| AppError::internal_error(e.to_string()))?;

    Ok(response)
}
