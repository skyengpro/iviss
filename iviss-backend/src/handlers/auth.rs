use crate::app_state::AppState;
use crate::dto::auth::{
    DailyLoginResponse, RequestDailyLoginRequest, RequestDailyLoginResponse,
    SendActivationResponse, VerifyDailyLoginRequest,
};
use crate::dto::users::{UserProfile, UserRole};
use crate::errors::AppError;
use crate::services::activation_service::ActivationService;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendActivationRequest {
    pub user_id: uuid::Uuid,
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "loginUser"
)]
pub async fn login(Json(_payload): Json<LoginRequest>) -> Result<impl IntoResponse, AppError> {
    // MOCK LOGIN
    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token: "mock-jwt-token".to_string(),
            user: UserProfile {
                id: uuid::Uuid::new_v4(),
                username: "admin".to_string(),
                email: Some("admin@iviss.com".to_string()),
                name: "Admin User".to_string(),
                role: UserRole::Admin,
                organization_id: uuid::Uuid::new_v4(),
                organization: Some("IVISS HQ".to_string()),
                badge_id: Some("ADMIN-01".to_string()),
                phone_number: Some("+237 600 000 000".to_string()),
                avatar_initials: Some("AU".to_string()),
                status: crate::dto::users::UserStatus::Active,
                is_active: true,
            },
        }),
    ))
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created", body = AuthResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "registerUser"
)]
pub async fn register(Json(payload): Json<RegisterRequest>) -> Result<impl IntoResponse, AppError> {
    // MOCK REGISTER
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token: "mock-jwt-token".to_string(),
            user: UserProfile {
                id: uuid::Uuid::new_v4(),
                username: payload
                    .email
                    .split('@')
                    .next()
                    .unwrap_or("user")
                    .to_string(),
                email: Some(payload.email),
                name: payload.full_name,
                role: payload.role,
                organization_id: uuid::Uuid::new_v4(),
                organization: Some("Independent".to_string()),
                badge_id: Some("TEMP-01".to_string()),
                phone_number: None,
                avatar_initials: Some("NU".to_string()),
                status: crate::dto::users::UserStatus::Active,
                is_active: true,
            },
        }),
    ))
}

/// Logout and invalidate session
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = String)
    ),
    tag = "auth",
    operation_id = "logoutUser",
    security(("bearer_auth" = []))
)]
pub async fn logout() -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json("Logout successful".to_string())))
}

/// Send activation code via SMS to a pending agent
#[utoipa::path(
    post,
    path = "/auth/send-activation",
    request_body = SendActivationRequest,
    responses(
        (status = 201, description = "Activation code sent", body = SendActivationResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "sendActivationCode"
)]
pub async fn send_activation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendActivationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch agent from DB
    let user = sqlx::query!(
        r#"
        SELECT id, phone_number,
               role AS "role: String",
               status AS "status: String"
        FROM users
        WHERE id = $1
        AND deleted_at IS NULL
        "#,
        payload.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // Only agents can receive an activation code
    if user.role != "agent" {
        return Err(AppError::BadRequest(
            "Activation is only available for agents".into(),
        ));
    }

    // Agent must be in PENDING_ACTIVATION or SUSPENDED status
    if user.status != "PENDING_ACTIVATION" && user.status != "SUSPENDED" {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation or suspended — current status: {}",
            user.status
        )));
    }

    // Build ActivationService from shared state resources
    let activation_svc = ActivationService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    // Generate, store and send the activation code via SMS
    activation_svc
        .generate_and_send(&user.id, &user.phone_number)
        .await
        .map_err(AppError::Internal)?;

    Ok((
        StatusCode::CREATED,
        Json(SendActivationResponse {
            message: "Activation code sent successfully".into(),
        }),
    ))
}

/// Request a daily OTP login code
#[utoipa::path(
    post,
    path = "/auth/request-daily-login",
    request_body = RequestDailyLoginRequest,
    responses(
        (status = 201, description = "OTP sent successfully", body = RequestDailyLoginResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 429, description = "Too many requests", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "requestDailyLogin"
)]
pub async fn request_daily_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestDailyLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    use crate::services::otp_service::OtpService;

    // Fetch user by phone number
    let user = sqlx::query!(
        r#"
        SELECT id, phone_number,
               role AS "role: String",
               status AS "status: String"
        FROM users
        WHERE phone_number = $1
        AND deleted_at IS NULL
        "#,
        payload.phone_number
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // Only active users can request OTP
    if user.status != "ACTIVE" {
        return Err(AppError::Unauthorized(format!(
            "Account is not active — current status: {}",
            user.status
        )));
    }

    // Verify device exists and is ACTIVE
    let device = sqlx::query!(
        r#"
        SELECT id, status AS "status: String"
        FROM devices
        WHERE id = $1
        AND user_id = $2
        "#,
        payload.device_id,
        user.id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Device not found".into()))?;

    if device.status != "ACTIVE" {
        return Err(AppError::Unauthorized(
            "Device is revoked or inactive".into(),
        ));
    }

    // Build OtpService and request OTP — rate limit enforced inside
    let otp_svc = OtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    otp_svc
        .request_otp(&user.id, &user.phone_number)
        .await
        .map_err(|e| {
            // Distinguish rate limit error from other errors
            if e.to_string().contains("Too many OTP requests") {
                AppError::BadRequest(e.to_string())
            } else {
                AppError::Internal(e)
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(RequestDailyLoginResponse {
            message: "OTP sent successfully".into(),
        }),
    ))
}

/// Verify daily OTP and issue shift token pair
#[utoipa::path(
    post,
    path = "/auth/verify-daily-login",
    request_body = VerifyDailyLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = DailyLoginResponse),
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
    use crate::services::otp_service::OtpService;
    use sha2::{Digest, Sha256};

    // Fetch user by phone number
    let user = sqlx::query!(
        r#"
        SELECT id, phone_number,
               status AS "status: String"
        FROM users
        WHERE phone_number = $1
        AND deleted_at IS NULL
        "#,
        payload.phone_number
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    // Verify device belongs to user and is ACTIVE
    let device = sqlx::query!(
        r#"
        SELECT id, status AS "status: String"
        FROM devices
        WHERE id = $1
        AND user_id = $2
        "#,
        payload.device_id,
        user.id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Device not found".into()))?;

    if device.status != "ACTIVE" {
        return Err(AppError::Unauthorized(
            "Device is revoked or inactive".into(),
        ));
    }

    // Validate OTP — handles expiry + attempts
    let otp_svc = OtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    otp_svc
        .validate_otp(&user.id, &payload.otp)
        .await
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    // Issue token pair via JwtService
    let token_pair = state
        .jwt_service
        .issue_token_pair(user.id, payload.device_id)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    // Hash refresh token before storing (SHA-256)
    let refresh_hash = format!("{:x}", Sha256::digest(token_pair.refresh_token.as_bytes()));

    // Store hashed refresh token linked to device
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, NOW() + INTERVAL '30 days')
        "#,
        refresh_hash,
        user.id,
        payload.device_id,
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::OK,
        Json(DailyLoginResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            shift_expires_at: token_pair.shift_expires_at,
        }),
    ))
}
