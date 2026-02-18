use crate::app_state::AppState;
use crate::dto::pending_submission::DataEntryResponse;
use crate::errors::AppError;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/vehicles/pending",
    tag = "vehicles",
    request_body = CreatePendingSubmissionRequest,
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
pub async fn submit_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<super::super::dto::pending_submission::CreatePendingSubmissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // In a real app we'd decode base64 images and upload to S3/Cloud storage here
    // For now we assume the frontend sends URLs or we just store the strings as-is (stub behavior for images)

    let submission_id = crate::queries::submission_queries::create_pending_submission(
        &state.db,
        payload.agent_id,
        payload.plate_number.clone(),
        payload.front_image_url,
        payload.back_image_url,
        payload.notes,
        payload.latitude,
        payload.longitude,
        None, // address not in DTO yet, pass allowed None
    )
    .await?;

    // Location fields are now passed to the query
    // match (payload.latitude, payload.longitude) { ... }

    let response = DataEntryResponse {
        message: "Submission accepted for review".to_string(),
        submission_id,
        plate_number: payload.plate_number,
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}
