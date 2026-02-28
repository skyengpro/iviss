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
    operation_id = "submitVehicle",
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

/// List all pending submissions for admin review
#[utoipa::path(
    get,
    path = "/admin/submissions",
    tag = "vehicles",
    operation_id = "listPendingSubmissions",
    responses(
        (status = 200, description = "List of pending submissions", body = [PendingSubmissionListItem]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_pending_submissions(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let submissions =
        crate::queries::submission_queries::get_pending_submissions(&state.db).await?;
    Ok((StatusCode::OK, Json(submissions)))
}

/// Get details of a single pending submission
#[utoipa::path(
    get,
    path = "/admin/submissions/{id}",
    tag = "vehicles",
    operation_id = "getPendingSubmission",
    params(
        ("id" = Uuid, Path, description = "Submission UUID")
    ),
    responses(
        (status = 200, description = "Submission details", body = PendingSubmissionRequest),
        (status = 404, description = "Submission not found", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_pending_submission(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let submission =
        crate::queries::submission_queries::get_submission_by_id(&state.db, id).await?;
    Ok((StatusCode::OK, Json(submission)))
}
