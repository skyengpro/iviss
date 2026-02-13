use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::dto::pending_submission::PendingSubmissionRequest;
use crate::errors::AppError;

#[utoipa::path(
    post,
    path = "/vehicles/pending",
    tag = "vehicles",
    request_body = PendingSubmissionRequest,
    responses(
        (status = 202, description = "Submission accepted for review", body = DataEntryResponse),
        (status = 400, description = "Invalid request",        body = AppError, 
            example = json!({ "code": "INVALID_REQUEST", "message": "Missing required field 'plate'" })),
        (status = 401, description = "Unauthorized",          body = AppError, 
            example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
        (status = 500, description = "Internal server error", body = AppError ,
             example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_vehicle(_payload: Json<PendingSubmissionRequest>) -> impl IntoResponse {
    // TODO:
    StatusCode::ACCEPTED
}
