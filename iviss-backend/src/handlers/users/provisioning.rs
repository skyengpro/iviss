use super::*;

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

    tracing::info!(
        user_id = %user.id,
        email = %user.email.as_deref().unwrap_or(""),
        "Org admin created successfully. Password sent via email."
    );

    // Send the password to the user's email
    state
        .email_svc
        .send_email(
            user.email.as_deref().unwrap_or(""),
            "org_admin",
            &temp_password,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ProvisionUserResponse {
            user,
            temp_password: None,
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
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(requester): Extension<AuthenticatedAdmin>,
) -> Result<impl IntoResponse, AppError> {
    let users = list_users_query(&state.db).await?;
    let filtered: Vec<_> = users
        .into_iter()
        .filter(|u| u.id != requester.user_id)
        .collect();
    Ok((StatusCode::OK, Json(filtered)))
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
    Extension(requester): Extension<AuthenticatedAdmin>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let target_user = get_user_by_id(&state.db, id).await?;

    if requester.role == "org_admin" {
        if target_user.role == UserRole::Admin {
            return Err(AppError::forbidden("Org admin cannot modify super admins"));
        }
        if target_user.organization_id != requester.organization_id {
            return Err(AppError::forbidden(
                "Org admin can only modify users within their organization",
            ));
        }
    }

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
    Extension(requester): Extension<AuthenticatedAdmin>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let target_user = get_user_by_id(&state.db, id).await?;

    if requester.role == "org_admin" {
        if target_user.role == UserRole::Admin {
            return Err(AppError::forbidden("Org admin cannot delete super admins"));
        }
        if target_user.organization_id != requester.organization_id {
            return Err(AppError::forbidden(
                "Org admin can only delete users within their organization",
            ));
        }
    }

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
    let filtered: Vec<_> = users
        .into_iter()
        .filter(|u| u.id != requester.user_id)
        .collect();
    Ok((StatusCode::OK, Json(filtered)))
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

    // Send activation OTP so the agent can activate their device
    let contact = if state.otp_via_email {
        user.email
            .clone()
            .unwrap_or_else(|| user.phone_number.clone().unwrap_or_default())
    } else {
        user.phone_number.clone().unwrap_or_default()
    };

    state
        .otp_svc
        .request_otp(&user.id, &contact)
        .await
        .map_err(|e| {
            tracing::error!(user_id = %user.id, error = %e, "Failed to send activation OTP");
            AppError::internal_error("User created but failed to send activation OTP")
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
