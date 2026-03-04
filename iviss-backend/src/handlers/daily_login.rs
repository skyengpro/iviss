use crate::app_state::AppState;
use crate::errors::AppError;
use crate::services::daily_otp_service::DailyOtpService;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

// ── Request / Response DTOs ──────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RequestDailyLoginRequest {
    pub phone_number: String,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RequestDailyLoginResponse {
    pub message: String,
    /// OTP validity in seconds (for the client countdown timer)
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyDailyLoginRequest {
    pub phone_number: String,
    pub otp: String,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyDailyLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    /// Token lifetime in seconds (8-hour shift)
    pub expires_in: u64,
    pub token_type: String,
}

// ── Internal row type for sqlx ───────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    phone_number: String,
    role: String,
    status: String,
}

// ── Handlers ─────────────────────────────────────────────────

/// Request a daily OTP at the start of a shift
#[utoipa::path(
    post,
    path = "/auth/request-daily-login",
    request_body = RequestDailyLoginRequest,
    responses(
        (status = 201, description = "Daily OTP sent via SMS", body = RequestDailyLoginResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "requestDailyLogin"
)]
pub async fn request_daily_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestDailyLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!(
        phone_number = %payload.phone_number,
        device_id = %payload.device_id,
        "Daily login OTP requested"
    );

    // Look up user by phone number and ensure they are ACTIVE
    let user = sqlx::query_as::<_, UserRow>(
        r#"SELECT id, phone_number, role::TEXT, status::TEXT
           FROM users
           WHERE phone_number = $1
           AND deleted_at IS NULL"#,
    )
    .bind(&payload.phone_number)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // User must be ACTIVE to request a daily OTP
    if user.status != "ACTIVE" {
        return Err(AppError::BadRequest(format!(
            "User is not active — current status: {}",
            user.status
        )));
    }

    // Build the daily OTP service from shared state
    let otp_svc = DailyOtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    // Generate, store, and send the OTP via SMS
    otp_svc
        .generate_and_send(&user.id, &user.phone_number)
        .await
        .map_err(AppError::Internal)?;

    Ok((
        StatusCode::CREATED,
        Json(RequestDailyLoginResponse {
            message: "Daily OTP sent successfully".into(),
            expires_in: 300, // 5 minutes
        }),
    ))
}

/// Verify the daily OTP and issue shift-scoped tokens
#[utoipa::path(
    post,
    path = "/auth/verify-daily-login",
    request_body = VerifyDailyLoginRequest,
    responses(
        (status = 200, description = "Shift tokens issued", body = VerifyDailyLoginResponse),
        (status = 401, description = "Invalid or expired OTP", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "verifyDailyLogin"
)]
pub async fn verify_daily_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyDailyLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Look up user by phone number
    let user = sqlx::query_as::<_, UserRow>(
        r#"SELECT id, phone_number, role::TEXT, status::TEXT
           FROM users
           WHERE phone_number = $1
           AND deleted_at IS NULL"#,
    )
    .bind(&payload.phone_number)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // Build OTP service
    let otp_svc = DailyOtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    // Validate the submitted OTP
    otp_svc
        .validate(&user.id, &payload.otp)
        .await
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    // TODO: Replace with real JWT generation when JWT service is implemented
    let shift_duration_secs: u64 = 8 * 60 * 60; // 8 hours

    Ok((
        StatusCode::OK,
        Json(VerifyDailyLoginResponse {
            access_token: format!("shift-jwt-{}", uuid::Uuid::new_v4()),
            refresh_token: format!("shift-refresh-{}", uuid::Uuid::new_v4()),
            expires_in: shift_duration_secs,
            token_type: "Bearer".into(),
        }),
    ))
}
