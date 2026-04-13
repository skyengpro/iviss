use crate::app_state::AppState;
use crate::dto::audit::{AuditLogListResponse, AuditLogQuery};
use crate::errors::AppError;
use crate::services::audit_service::AuditService;
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
        (status = 401, description = "Unauthorized",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",     body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "listAuditLogs",
    security(("bearer_auth" = []))
)]
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (items, total) = AuditService::list(&state.db, &query).await?;

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
        (status = 403, description = "Forbidden",    body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "exportAuditLogs",
    security(("bearer_auth" = []))
)]
pub async fn export_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let items = AuditService::export(&state.db, &query).await?;
    let csv = AuditService::build_csv(&items);

    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"audit_logs.csv\"",
        ),
    ];

    Ok((StatusCode::OK, headers, csv))
}

/// Export audit logs as PDF (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs/export-pdf",
    params(AuditLogQuery),
    responses(
        (status = 200, description = "PDF file", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden",    body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "exportAuditLogsPdf",
    security(("bearer_auth" = []))
)]
pub async fn export_audit_logs_pdf(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let items = AuditService::export(&state.db, &query).await?;
    let pdf_bytes = AuditService::build_pdf(items).await?;

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"audit_logs.pdf\"",
        ),
    ];

    Ok((StatusCode::OK, headers, pdf_bytes))
}
