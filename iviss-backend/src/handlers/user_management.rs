use crate::app_state::AppState;
use crate::dto::users::{ProvisionUserRequest, UpdateUserRequest};
use crate::errors::AppError;
use crate::queries::organization_queries::list_organizations as list_organizations_query;
use crate::queries::user_queries::{
    create_user, get_user_by_id, hard_delete_user, list_users as list_users_query,
    update_user as update_user_query,
};
use crate::services::activation_service::ActivationService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
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
            let redis = state.redis.clone();
            let sms = state.sms_pvd.clone();
            let pepper = state.pepper.clone();
            tokio::spawn(async move {
                let svc = ActivationService::new(redis, sms, pepper);
                if let Err(e) = svc.generate_and_send(&user_id, &phone).await {
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminateSessionRequest {
    pub user_id: Uuid,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TerminateSessionResponse {
    pub message: String,
}

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
