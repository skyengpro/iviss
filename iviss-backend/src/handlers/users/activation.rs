use super::*;

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
    Extension(requester): Extension<AuthenticatedAdmin>,
    Json(payload): Json<ResendActivationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch agent from DB
    let user_raw = sqlx::query(
        r#"
        SELECT u.id,
               u.phone_number,
               u.role,
               u.status,
               u.organization_id,
               d.status AS device_status
        FROM users u
        LEFT JOIN (
            SELECT DISTINCT ON (user_id)
                user_id, status
            FROM devices
            ORDER BY user_id, updated_at DESC
        ) d ON u.id = d.user_id
        WHERE u.id = $1
          AND u.deleted_at IS NULL
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
    let organization_id: Option<Uuid> = user_raw.get("organization_id");
    let device_status: Option<crate::dto::users::DeviceStatus> = user_raw.get("device_status");

    if requester.role == "org_admin" && requester.organization_id != organization_id {
        return Err(AppError::forbidden(
            "Org admin can only resend activation codes for users in their organization",
        ));
    }

    // Only agents can receive an activation code
    if role != UserRole::Agent {
        return Err(AppError::BadRequest(
            "Activation is only available for agents".into(),
        ));
    }

    let device_requires_reactivation = matches!(
        device_status,
        Some(crate::dto::users::DeviceStatus::Suspended)
    );

    if status != UserStatus::PendingActivation && !device_requires_reactivation {
        return Err(AppError::BadRequest(format!(
            "Activation code can only be resent for pending agents or agents whose device requires reactivation — current user status: {}, device status: {}",
            status.as_str(),
            device_status.map(|s| s.as_str()).unwrap_or("NONE")
        )));
    }

    if status != UserStatus::PendingActivation {
        let mut tx = state.db.begin().await.map_err(AppError::Database)?;

        sqlx::query(
            r#"
            UPDATE users
            SET status = 'PENDING_ACTIVATION'::user_status
            WHERE id = $1
              AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE,
                revoked_at = NOW()
            WHERE user_id = $1
              AND revoked = FALSE
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        tx.commit().await.map_err(AppError::Database)?;
    }

    // Build OtpService from shared state resources
    let otp_svc = &state.otp_svc;

    // Determine contact based on AppState setting
    let contact = if state.otp_via_email {
        let profile = crate::queries::user_queries::get_user_by_id(&state.db, user_id).await?;
        profile
            .email
            .clone()
            .unwrap_or_else(|| profile.phone_number.clone().unwrap_or_default())
    } else {
        phone_number.clone()
    };

    // Generate, store and send the activation code
    otp_svc.request_otp(&user_id, &contact).await?;

    Ok((
        StatusCode::CREATED,
        Json(ResendActivationResponse {
            message: "Activation code sent successfully".into(),
        }),
    ))
}

/// Resend temporary password to a pending org admin
#[utoipa::path(
    post,
    path = "/api/v1/admin/resend-org-admin-password",
    request_body = ResendOrgAdminPasswordRequest,
    responses(
        (status = 201, description = "Password sent successfully", body = ResendOrgAdminPasswordResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 403, description = "Forbidden", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "resendOrgAdminPassword",
    security(("bearer_auth" = []))
)]
pub async fn resend_org_admin_password(
    State(state): State<Arc<AppState>>,
    Extension(requester): Extension<AuthenticatedAdmin>,
    Json(payload): Json<ResendOrgAdminPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Only superadmin can resend org admin passwords
    if requester.role != "admin" {
        return Err(AppError::forbidden(
            "Only superadmin can resend org admin passwords",
        ));
    }

    // Fetch org admin from DB
    let user_raw = sqlx::query(
        r#"
        SELECT id, email, role, status
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
    let email: Option<String> = user_raw.get("email");
    let role: UserRole = user_raw.get("role");
    let status: UserStatus = user_raw.get("status");

    // Only org admins can receive a password resend
    if role != UserRole::OrgAdmin {
        return Err(AppError::BadRequest(
            "Password resend is only available for org admins".into(),
        ));
    }

    // Only pending activation org admins can receive a password resend
    if status != UserStatus::PendingActivation {
        return Err(AppError::BadRequest(format!(
            "Password can only be resent for org admins with pending activation status — current status: {}",
            status.as_str()
        )));
    }

    let email = email.ok_or_else(|| AppError::bad_request("User must have an email"))?;

    // Generate new temporary password
    let temp_password =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes());
    let password_hash = crate::utils::password::hash_password(&temp_password).await?;

    // Update password hash in database
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1,
            must_change_password = TRUE
        WHERE id = $2
          AND deleted_at IS NULL
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    tracing::info!(
        user_id = %user_id,
        email = %email,
        "Org admin password resent successfully"
    );

    // Send the password to the user's email
    state
        .email_svc
        .send_email(&email, "org_admin", &temp_password)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ResendOrgAdminPasswordResponse {
            message: "Password sent successfully".into(),
        }),
    ))
}
