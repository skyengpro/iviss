use crate::app_state::AppState;
use crate::errors::AppError;
use crate::queries::auth_queries;
use axum::extract::{Path, State};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceActionResponse {
    pub message: String,
}

// ── Suspend
#[utoipa::path(
    post,
    path = "/admin/devices/{id}/suspend",
    params(
        ("id" = Uuid, Path, description = "Device UUID to suspend")
    ),
    responses(
        (status = 200, description = "Device suspended", body = DeviceActionResponse),
        (status = 400, description = "Device is already suspended", body = AppErrorResponse),
        (status = 404, description = "Device not found", body = AppErrorResponse),
    ),
    tag = "admin",
    operation_id = "suspendDevice",
    security(("bearer_auth" = []))
)]
pub async fn suspend_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // ── Verify device exists and is not already suspended
    let row = sqlx::query(
        r#"
        SELECT status::TEXT AS status
        FROM devices
        WHERE id = $1
        "#,
    )
    .bind(device_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::not_found("Device not found"))?;

    let status: String = sqlx::Row::get(&row, "status");

    if status == "SUSPENDED" {
        return Err(AppError::bad_request("Device is already suspended"));
    }

    auth_queries::revoke_refresh_tokens_for_device(&state.db, device_id).await?;

    // ── Set device status to SUSPENDED
    auth_queries::mark_device_suspended(&state.db, device_id).await?;

    tracing::info!(
        target: "device_management",
        device_id = %device_id,
        "Device suspended by admin"
    );

    Ok((
        StatusCode::OK,
        Json(DeviceActionResponse {
            message: "Device suspended successfully".into(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/admin/devices/{id}/unsuspend",
    params(
        ("id" = Uuid, Path, description = "Device UUID to unsuspend")
    ),
    responses(
        (status = 200, description = "Device restored to INACTIVE", body = DeviceActionResponse),
        (status = 400, description = "Device is not suspended", body = AppErrorResponse),
        (status = 404, description = "Device not found", body = AppErrorResponse),
    ),
    tag = "admin",
    operation_id = "unsuspendDevice",
    security(("bearer_auth" = []))
)]
pub async fn unsuspend_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // ── Verify device exists and is currently SUSPENDED
    let row = sqlx::query(
        r#"
        SELECT status::TEXT AS status
        FROM devices
        WHERE id = $1
        "#,
    )
    .bind(device_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::not_found("Device not found"))?;

    let status: String = sqlx::Row::get(&row, "status");

    if status != "SUSPENDED" {
        return Err(AppError::bad_request(format!(
            "Device is not suspended — current status: {}",
            status
        )));
    }

    auth_queries::mark_device_inactive(&state.db, device_id).await?;

    tracing::info!(
        target: "device_management",
        device_id = %device_id,
        "Device suspension lifted by admin — status set to INACTIVE"
    );

    Ok((
        StatusCode::OK,
        Json(DeviceActionResponse {
            message: "Device suspension lifted — agent can now log in again".into(),
        }),
    ))
}
