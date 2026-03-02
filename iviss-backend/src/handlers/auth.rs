use crate::app_state::AppState;
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendActivationResponse {
    pub message: String,
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
                email: "admin@iviss.com".to_string(),
                name: "Admin User".to_string(),
                role: UserRole::Admin,
                organization_id: uuid::Uuid::new_v4(),
                organization: Some("IVISS HQ".to_string()),
                badge_id: Some("ADMIN-01".to_string()),
                phone_number: Some("+237 600 000 000".to_string()),
                avatar_initials: Some("AU".to_string()),
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
                email: payload.email,
                name: payload.full_name,
                role: payload.role,
                organization_id: uuid::Uuid::new_v4(),
                organization: Some("Independent".to_string()),
                badge_id: Some("TEMP-01".to_string()),
                phone_number: None,
                avatar_initials: Some("NU".to_string()),
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

    // Agent must be in PENDING_ACTIVATION status
    if user.status != "PENDING_ACTIVATION" {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation — current status: {}",
            user.status
        )));
    }

    // Build ActivationService from shared state resources
    let activation_svc = ActivationService::new(state.redis.clone(), state.sms_pvd.clone());

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
