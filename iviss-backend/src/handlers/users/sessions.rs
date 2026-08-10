use super::*;

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

    crate::queries::auth::sessions::terminate_user_sessions(&state.db, payload.user_id).await?;

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
    crate::queries::auth::sessions::restart_user_session(
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
