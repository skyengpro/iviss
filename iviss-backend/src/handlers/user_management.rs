use crate::app_state::AppState;
use crate::dto::users::{
    ProvisionUserRequest, ProvisionUserResponse, ResendActivationRequest, ResendActivationResponse,
    RestartSessionRequest, RestartSessionResponse, TerminateSessionRequest,
    TerminateSessionResponse, UpdateUserRequest,
};
use crate::dto::users::{UserRole, UserStatus};
use crate::errors::AppError;
use crate::middleware::rbac::AuthenticatedAdmin;
use crate::queries::organization_queries::list_organizations as list_organizations_query;
use crate::queries::user_queries::{
    create_org_admin_user_with_temp_password, get_user_by_id, hard_delete_user,
    list_users as list_users_query, list_users_by_org, update_user as update_user_query,
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::Engine;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

/// Provision a new user (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/admin/users",
    request_body = ProvisionUserRequest,
    responses(
        (status = 201, description = "User provisioned successfully", body = ProvisionUserResponse),
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
    Extension(requester): Extension<AuthenticatedAdmin>,
    Json(payload): Json<ProvisionUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    if requester.role != "admin" {
        return Err(AppError::forbidden(
            "Only superadmin can provision organization admins",
        ));
    }

    if payload.email.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::bad_request("email is required"));
    }

    let mut req = payload;
    req.role = UserRole::OrgAdmin;

    let temp_password =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes());
    let password_hash = crate::utils::password::hash_password(&temp_password).await?;

    let user = create_org_admin_user_with_temp_password(&state.db, req, password_hash).await?;

    println!(
        "\n╔══════════════════════════════════════════════════╗\
         \n║        ORG ADMIN ACCOUNT CREATED                 ║\
         \n╠══════════════════════════════════════════════════╣\
         \n║  Email    : {:<38}║\
         \n║  Password : {:<38}║\
         \n║  User ID  : {:<38}║\
         \n╚══════════════════════════════════════════════════╝\
         \n  ⚠  Share these credentials securely.\
         \n  ⚠  The user must change the password on first login.\n",
        user.email.as_deref().unwrap_or("(none)"),
        temp_password,
        user.id,
    );

    tracing::info!(
        user_id = %user.id,
        email = %user.email.as_deref().unwrap_or(""),
        "Org admin created successfully"
    );

    // Send the password to the user's email
    state.email_svc.send_email(user.email.as_deref().unwrap_or(""), &temp_password).await?;

    Ok((
        StatusCode::CREATED,
        Json(ProvisionUserResponse {
            user,
            temp_password: Some(temp_password),
        }),
    ))
}

/// List all users (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
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
    path = "/api/v1/admin/users/{id}",
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
    path = "/api/v1/admin/users/{id}",
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
    path = "/api/v1/admin/users/{id}",
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
    path = "/api/v1/admin/organizations",
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
    path = "/api/v1/admin/terminate-session",
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
    path = "/api/v1/admin/restart-session",
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
    path = "/api/v1/admin/resend-activation-code",
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

/// List users scoped to the org admin's organization
#[utoipa::path(
    get,
    path = "/api/v1/org-admin/users",
    responses(
        (status = 200, description = "Users retrieved successfully", body = [UserProfile]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden",    body = AppErrorResponse)
    ),
    tag = "org-admin",
    operation_id = "listOrgUsers",
    security(("bearer_auth" = []))
)]
pub async fn list_org_users(
    State(state): State<Arc<AppState>>,
    Extension(requester): Extension<AuthenticatedAdmin>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = requester
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;
    let users = list_users_by_org(&state.db, org_id).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// Create an agent or supervisor within the org admin's organization
#[utoipa::path(
    post,
    path = "/api/v1/org-admin/users",
    request_body = ProvisionUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = ProvisionUserResponse),
        (status = 400, description = "Bad request",  body = AppErrorResponse),
        (status = 403, description = "Forbidden",    body = AppErrorResponse)
    ),
    tag = "org-admin",
    operation_id = "provisionOrgUser",
    security(("bearer_auth" = []))
)]
pub async fn provision_org_user(
    State(state): State<Arc<AppState>>,
    Extension(requester): Extension<AuthenticatedAdmin>,
    Json(payload): Json<ProvisionUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = requester
        .organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?;

    if !matches!(payload.role, UserRole::Agent | UserRole::Manager) {
        return Err(AppError::forbidden(
            "Org admin can only create agents or supervisors",
        ));
    }

    let mut req = payload;
    req.organization_id = org_id;

    // phone_number is required to send the activation OTP
    if req.phone_number.trim().is_empty() {
        return Err(AppError::bad_request("phone_number is required"));
    }

    let user = crate::queries::user_queries::create_user(&state.db, req).await?;

    // Send activation OTP via SMS so the agent can activate their device
    state
        .otp_svc
        .request_otp(&user.id, user.phone_number.as_deref().unwrap_or(""))
        .await
        .map_err(|e| {
            tracing::error!(user_id = %user.id, error = %e, "Failed to send activation OTP");
            AppError::internal_error("User created but failed to send activation SMS")
        })?;

    tracing::info!(
        user_id = %user.id,
        org_id = %org_id,
        "User created by org admin and activation OTP sent"
    );

    Ok((
        StatusCode::CREATED,
        Json(ProvisionUserResponse {
            user,
            temp_password: None,
        }),
    ))
}
