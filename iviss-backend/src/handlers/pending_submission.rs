use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::dto::pending_submission::PendingSubmissionRequest;
use crate::errors::{AppError, AppErrorResponse};

#[utoipa::path(
    post,
    path = "/vehicles/pending",
    tag = "vehicles",
    request_body = PendingSubmissionRequest,
    responses(
        (status = 202, description = "Submission accepted for review", body = DataEntryResponse),
        (status = 400, description = "Invalid request",        body = AppErrorResponse, 
            example = json!({ "code": "INVALID_REQUEST", "message": "Missing required field 'plate'" })),
        (status = 401, description = "Unauthorized",          body = AppErrorResponse, 
            example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
        (status = 500, description = "Internal server error", body = AppErrorResponse ,
             example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_vehicle(_payload: Json<PendingSubmissionRequest>) -> impl IntoResponse {
    // TODO:
    StatusCode::ACCEPTED
}
