use super::*;

/// Change password for admin/org_admin users (used for forced password change on first login)
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = ChangePasswordResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
    ),
    tag = "auth",
    operation_id = "changePassword",
    security(("bearer_auth" = []))
)]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(requester): Extension<AuthenticatedAdmin>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.new_password != payload.confirm_password {
        return Err(AppError::bad_request("Passwords do not match"));
    }

    if payload.new_password.trim().is_empty() {
        return Err(AppError::bad_request("New password cannot be empty"));
    }

    // Fetch user's current state to determine if current password is required
    let user = auth::get_password_change_state(&state.db, requester.user_id).await?;

    let (user_password_hash, must_change_password) = user;

    // Security: Current password is REQUIRED unless this is a forced first-login password change
    if !must_change_password {
        // Normal password change - current password is MANDATORY
        let current_password = payload.current_password.as_ref().ok_or_else(|| {
            AppError::bad_request("Current password is required for password changes")
        })?;

        // Verify current password
        let is_valid =
            crate::utils::password::verify_password(current_password, &user_password_hash)
                .await
                .map_err(|_| AppError::bad_request("Invalid current password"))?;

        if !is_valid {
            return Err(AppError::bad_request("Invalid current password"));
        }
    }
    // If must_change_password is true (first login), current password is optional

    let new_hash = crate::utils::password::hash_password(&payload.new_password).await?;

    auth::update_password_after_change(&state.db, requester.user_id, &new_hash).await?;

    tracing::info!(user_id = %requester.user_id, "Password changed successfully");

    Ok((
        StatusCode::OK,
        Json(ChangePasswordResponse {
            message: "Password changed successfully".to_string(),
        }),
    ))
}
