use crate::app_state::AppState;
use crate::middleware::auth::AuthenticatedUser;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use std::sync::Arc;

use crate::errors::AppError;

// ── GET /users/me ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/users/me",
    tag = "users",
    operation_id = "getUserProfile",
    responses(
        (status = 200, description = "Current user profile", body = UserProfile),
        (status = 401, description = "Unauthorized",         body = AppErrorResponse, example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
        (status = 500, description = "Internal server error",body = AppErrorResponse , example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" }))),
    security(("bearer_auth" = []))
)]
pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, AppError> {
    let _ = (&auth.device_id, &auth.jti);
    let profile = crate::queries::user_queries::get_user_by_id(&state.db, auth.user_id).await?;
    Ok((StatusCode::OK, Json(profile)))
}
