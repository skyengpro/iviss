use crate::app_state::AppState;
use crate::dto::users::{
    ProvisionUserRequest, ResendActivationRequest, ResendActivationResponse, RestartSessionRequest,
    RestartSessionResponse, TerminateSessionRequest, TerminateSessionResponse, UpdateUserRequest,
};
use crate::dto::users::{UserRole, UserStatus};
use crate::errors::AppError;
use crate::queries::organization_queries::list_organizations as list_organizations_query;
use crate::queries::user_queries::{
    create_user, get_user_by_id, hard_delete_user, list_users as list_users_query,
    update_user as update_user_query,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::Row;
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

/// Provision a new user (admin only)
#[utoipa::path(
    post,
    path = "/admin/users",
    request_body = ProvisionUserRequest,
    responses(
        (status = 201, description = "User provisioned successfully", body = UserProfile),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "provisionUser",
    security(("bearer_auth" = []))
)]
pub async fn provision_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProvisionUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = create_user(&state.db, payload).await?;

    // If user is an agent, send activation code
    if matches!(user.role, crate::dto::users::UserRole::Agent) {
        if let Some(phone) = user.phone_number.clone() {
            let user_id = user.id;
            let otp_svc = state.otp_svc.clone();

            tokio::spawn(async move {
                if let Err(e) = otp_svc.request_otp(&user_id, &phone).await {
                    warn!("Failed to send activation code to {}: {}", phone, e);
                }
            });
        }
    }

    tracing::info!("User created successfully: {}", user.id);

    Ok((StatusCode::CREATED, Json(user)))
}

/// List all users (admin only)
#[utoipa::path(
    get,
    path = "/admin/users",
    responses(
        (status = 200, description = "Users retrieved successfully", body = [UserProfile]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "listUsers",
    security(("bearer_auth" = []))
)]
pub async fn list_users(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let users = list_users_query(&state.db).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// Get a specific user by ID (admin only)
#[utoipa::path(
    get,
    path = "/admin/users/{id}",
    responses(
        (status = 200, description = "User retrieved successfully", body = UserProfile),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "getUser",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = get_user_by_id(&state.db, id).await?;
    Ok((StatusCode::OK, Json(user)))
}

/// Update a user (admin only)
#[utoipa::path(
    put,
    path = "/admin/users/{id}",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserProfile),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "updateUser",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = update_user_query(&state.db, id, payload).await?;
    Ok((StatusCode::OK, Json(user)))
}

/// Delete a user (admin only)
#[utoipa::path(
    delete,
    path = "/admin/users/{id}",
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "deleteUser",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    hard_delete_user(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List all organizations (admin only)
#[utoipa::path(
    get,
    path = "/admin/organizations",
    responses(
        (status = 200, description = "Organizations retrieved successfully", body = [Organization]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "listOrganizations",
    security(("bearer_auth" = []))
)]
pub async fn list_organizations(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let orgs = list_organizations_query(&state.db).await?;
    Ok((StatusCode::OK, Json(orgs)))
}

// ── Session Termination ──

/// Terminate all sessions for a user (admin only).
///
/// Revokes all refresh tokens, deactivates all devices, and suspends
/// the user account. The next request from that user will return 401.
#[utoipa::path(
    post,
    path = "/admin/terminate-session",
    request_body = TerminateSessionRequest,
    responses(
        (status = 200, description = "Session terminated", body = TerminateSessionResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "terminateSession",
    security(("bearer_auth" = []))
)]
pub async fn terminate_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TerminateSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify target user exists and is an agent
    let user = get_user_by_id(&state.db, payload.user_id).await?;

    if !matches!(user.role, crate::dto::users::UserRole::Agent) {
        return Err(AppError::bad_request(
            "Session termination is only available for agents",
        ));
    }

    crate::queries::session_queries::terminate_user_sessions(&state.db, payload.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(TerminateSessionResponse {
            message: format!("All sessions terminated for user {}", payload.user_id),
        }),
    ))
}

/// Restarts/Extends a session for an agent
#[utoipa::path(
    post,
    path = "/admin/restart-session",
    request_body = RestartSessionRequest,
    responses(
        (status = 200, description = "Session restarted", body = RestartSessionResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "restartSession",
    security(("bearer_auth" = []))
)]
pub async fn restart_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RestartSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify target user exists and is an agent
    let user = get_user_by_id(&state.db, payload.user_id).await?;

    if !matches!(user.role, crate::dto::users::UserRole::Agent) {
        return Err(AppError::bad_request(
            "Session restart is only available for agents",
        ));
    }

    // Refresh the device status and shift_end (default to 8 hours for restart)
    crate::queries::session_queries::restart_user_session(
        &state.db,
        payload.user_id,
        std::time::Duration::from_secs(8 * 3600),
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(RestartSessionResponse {
            message: format!("Session restarted for user {}", payload.user_id),
        }),
    ))
}

/// Resend activation code via SMS to a pending agent
#[utoipa::path(
    post,
    path = "/admin/resend-activation-code",
    request_body = ResendActivationRequest,
    responses(
        (status = 201, description = "Activation code sent", body = ResendActivationResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "resendActivationCode"
)]
pub async fn resend_activation_code(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResendActivationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch agent from DB
    let user_raw = sqlx::query(
        r#"
        SELECT id,
               phone_number,
               role,
               status
        FROM users
        WHERE id = $1
        AND deleted_at IS NULL
        "#,
    )
    .bind(payload.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let user_id: Uuid = user_raw.get("id");
    let phone_number: String = user_raw.get("phone_number");
    let role: UserRole = user_raw.get("role");
    let status: UserStatus = user_raw.get("status");

    // Only agents can receive an activation code
    if role != UserRole::Agent {
        return Err(AppError::BadRequest(
            "Activation is only available for agents".into(),
        ));
    }

    // Agent must be in PENDING_ACTIVATION status
    if status != UserStatus::PendingActivation {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation — current status: {}",
            status.as_str()
        )));
    }

    // Build OtpService from shared state resources
    let otp_svc = &state.otp_svc;

    // Generate, store and send the activation code via SMS
    otp_svc.request_otp(&user_id, &phone_number).await?;

    Ok((
        StatusCode::CREATED,
        Json(ResendActivationResponse {
            message: "Activation code sent successfully".into(),
        }),
    ))
}
