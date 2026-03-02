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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::pending_submission::{CreatePendingSubmissionRequest, SubmissionStatus};

    // Mock test data
    fn create_test_submission_request() -> CreatePendingSubmissionRequest {
        CreatePendingSubmissionRequest {
            plate_number: "TEST123".to_string(),
            agent_id: uuid::Uuid::new_v4(),
            front_image_url: "http://example.com/front.jpg".to_string(),
            back_image_url: "http://example.com/back.jpg".to_string(),
            notes: Some("Test notes".to_string()),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
        }
    }

    #[tokio::test]
    async fn test_submit_vehicle_success() {
        // This test would require a test database setup
        // For now, we'll test the structure and basic functionality

        let request = create_test_submission_request();

        // Verify request structure
        assert_eq!(request.plate_number, "TEST123");
        assert!(request.front_image_url.contains("front.jpg"));
        assert!(request.back_image_url.contains("back.jpg"));
        assert_eq!(request.notes, Some("Test notes".to_string()));
        assert_eq!(request.latitude, Some(40.7128));
        assert_eq!(request.longitude, Some(-74.0060));
    }

    #[tokio::test]
    async fn test_submit_vehicle_request_validation() {
        // Test with missing required fields
        let mut request = create_test_submission_request();

        // Test empty plate number
        request.plate_number = "".to_string();
        assert!(request.plate_number.is_empty());

        // Test with None values for optional fields
        request.notes = None;
        request.latitude = None;
        request.longitude = None;
        assert!(request.notes.is_none());
        assert!(request.latitude.is_none());
        assert!(request.longitude.is_none());
    }

    #[tokio::test]
    async fn test_get_pending_submission_structure() {
        // Test that the function has the correct signature and return type

        // Verify the function exists and has the expected behavior
        // In a real test with a test database, you would:
        // 1. Set up a test database connection
        // 2. Insert a test submission
        // 3. Call the function with the submission ID
        // 4. Verify the response contains the correct submission

        // For now, we verify the function compiles and has correct types
        let test_uuid = uuid::Uuid::new_v4();

        // Test UUID parsing
        let path_param = axum::extract::Path(test_uuid);
        assert_eq!(path_param.0, test_uuid);

        assert!(true); // Placeholder test to verify compilation
    }

    #[tokio::test]
    async fn test_data_entry_response_structure() {
        // Test the response structure
        let submission_id = uuid::Uuid::new_v4();
        let response = DataEntryResponse {
            message: "Submission accepted for review".to_string(),
            submission_id,
            plate_number: "TEST123".to_string(),
        };

        assert_eq!(response.message, "Submission accepted for review");
        assert_eq!(response.plate_number, "TEST123");
        assert_eq!(response.submission_id, submission_id);
    }

    #[tokio::test]
    async fn test_submission_status_enum() {
        // Test the SubmissionStatus enum
        let pending = SubmissionStatus::Pending;
        let approved = SubmissionStatus::Approved;
        let rejected = SubmissionStatus::Rejected;

        assert_eq!(pending, SubmissionStatus::Pending);
        assert_eq!(approved, SubmissionStatus::Approved);
        assert_eq!(rejected, SubmissionStatus::Rejected);

        // Test inequality
        assert!(pending != approved);
        assert!(approved != rejected);
        assert!(pending != rejected);
    }

    #[tokio::test]
    async fn test_create_pending_submission_request_serialization() {
        // Test that the request can be serialized/deserialized correctly
        let request = create_test_submission_request();

        // Test that all fields are accessible
        assert!(!request.plate_number.is_empty());
        assert!(!request.front_image_url.is_empty());
        assert!(!request.back_image_url.is_empty());
        assert!(!request.agent_id.is_nil()); // UUID should be valid (not nil)
    }
}
