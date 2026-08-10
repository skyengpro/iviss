use super::*;

// ── POST /users/location ───────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/users/location",
    tag = "users",
    request_body = UpdateLocationRequest,
    operation_id = "updateLocation",
    responses(
        (status = 200, description = "Location updated successfully", body = UpdateLocationResponse),
        (status = 401, description = "Unauthorized",         body = AppErrorResponse),
        (status = 500, description = "Internal server error",body = AppErrorResponse)
    )
)]
pub async fn update_location(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateLocationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if auth.role != "agent" {
        return Err(AppError::forbidden("Only agents can update location"));
    }

    crate::queries::users::location::update_agent_location_query(
        &state.db,
        auth.user_id,
        payload.latitude,
        payload.longitude,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(UpdateLocationResponse {
            message: "Location updated".to_string(),
        }),
    ))
}
