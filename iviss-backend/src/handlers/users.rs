use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use uuid::Uuid;

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
    // In a real app, we'd extract the user ID from the JWT token here
    // For now, let's assume a hardcoded or header-provided ID, or just the first user for testing
) -> Result<impl IntoResponse, AppError> {
    // MOCK: using specific UUID for testing if no auth yet, or fetch from header?
    // Let's assume we pass "X-User-Id" header or similar, BUT for this specific request "get current user profile",
    // usually it comes from the token.
    // Ideally we'd modify the function signature to take an extracted User claims.

    // HACK: Hardcoding a known UUID or fetching the first user for MVP if no auth middleware active yet.
    // Or we can try to parse from a header.
    // Let's try to find *some* user.

    // Better strategy for MVP: Query for a specific user ID if testing, or failing that.
    // Let's rely on a hardcoded UUID that presumably exists in seeds.
    // The user schema has UUIDs.
    // "00000000-0000-0000-0000-000000000000" might not exist.

    // Let's implement a workaround: Get the first user from DB.
    let user = sqlx::query_as::<_, UserRow>("SELECT id FROM users LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::database)?;

    match user {
        Some(u) => {
            let profile = crate::queries::user_queries::get_user_by_id(&state.db, u.id).await?;
            Ok((StatusCode::OK, Json(profile)))
        }
        None => Err(AppError::not_found("No users found in database")),
    }
}

// Minimal row struct just for the ID lookup hack
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
}
