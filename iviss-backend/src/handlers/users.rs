use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dto::users::{UserProfile, UserRole},
    errors::{AppError, AppErrorResponse},
};

// ── GET /users/me ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/users/me",
    tag = "users",
    responses(
        (status = 200, description = "Current user profile", body = UserProfile),
        (status = 401, description = "Unauthorized",         body = AppErrorResponse, example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
        (status = 500, description = "Internal server error",body = AppErrorResponse , example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" }))),
    security(("bearer_auth" = []))
)]
pub async fn get_user_profile() -> impl IntoResponse {
    // TODO:
    StatusCode::ACCEPTED
}
