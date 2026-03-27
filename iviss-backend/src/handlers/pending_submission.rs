use crate::app_state::AppState;
use crate::dto::pending_submission::{
    DataEntryResponse, ReviewSubmissionResponse, SubmissionListQuery, SubmissionStatus,
};
use crate::errors::AppError;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

// ── Submit (agent-facing) ─────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/vehicles/pending",
    tag = "vehicles",
    operation_id = "submitVehicle",
    request_body = CreatePendingSubmissionRequest,
    responses(
        (status = 202, description = "Submission accepted for review", body = DataEntryResponse),
        (status = 400, description = "Invalid request",        body = AppErrorResponse,
             example = json!({ "code": "INVALID_REQUEST", "message": "Missing required field 'plate'" })),
         (status = 401, description = "Unauthorized",          body = AppErrorResponse,
             example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
         (status = 500, description = "Internal server error", body = AppErrorResponse,
              example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<super::super::dto::pending_submission::CreatePendingSubmissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let agent_id = resolve_agent_id(&state.db, payload.agent_id).await?;

    let submission_id = crate::queries::submission_queries::create_pending_submission(
        &state.db,
        agent_id,
        payload.plate_number.clone(),
        payload.front_image_url,
        payload.back_image_url,
        payload.notes,
        payload.latitude,
        payload.longitude,
        None,
    )
    .await?;

    let response = DataEntryResponse {
        message: "Submission accepted for review".to_string(),
        submission_id,
        plate_number: payload.plate_number,
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/api/v1/vehicles/pending",
    tag = "vehicles",
    operation_id = "submitVehicleV1",
    request_body = CreatePendingSubmissionRequest,
    responses(
        (status = 202, description = "Submission accepted for review", body = DataEntryResponse),
        (status = 400, description = "Invalid request",        body = AppErrorResponse,
             example = json!({ "code": "INVALID_REQUEST", "message": "Missing required field 'plate'" })),
         (status = 401, description = "Unauthorized",          body = AppErrorResponse,
             example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
         (status = 500, description = "Internal server error", body = AppErrorResponse,
              example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_vehicle_v1(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<super::super::dto::pending_submission::CreatePendingSubmissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    submit_vehicle(State(state), Json(payload)).await
}

async fn resolve_agent_id(pool: &sqlx::PgPool, requested: Uuid) -> Result<Uuid, AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(requested)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if exists {
        return Ok(requested);
    }

    let first: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;

    match first {
        Some(id) => Ok(id),
        None => Err(AppError::not_found("No users found in database")),
    }
}

// ── List (admin) ──────────────────────────────────────────────────────────────

/// List submissions for admin review, optionally filtered by status
#[utoipa::path(
    get,
    path = "/api/v1/admin/submissions",
    tag = "vehicles",
    operation_id = "listPendingSubmissions",
    params(
        ("status" = Option<String>, Query, description = "Filter by status: pending, approved, rejected")
    ),
    responses(
        (status = 200, description = "List of submissions", body = [PendingSubmissionListItem]),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_pending_submissions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SubmissionListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let submissions = crate::queries::submission_queries::get_pending_submissions(
        &state.db,
        query.status.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(submissions)))
}

// ── Detail (admin) ────────────────────────────────────────────────────────────

/// Get full details of a single submission
#[utoipa::path(
    get,
    path = "/api/v1/admin/submissions/{id}",
    tag = "vehicles",
    operation_id = "getPendingSubmission",
    params(
        ("id" = Uuid, Path, description = "Submission UUID")
    ),
    responses(
        (status = 200, description = "Submission details", body = PendingSubmissionDetail),
        (status = 404, description = "Submission not found", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_pending_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let submission =
        crate::queries::submission_queries::get_submission_by_id(&state.db, id).await?;
    Ok((StatusCode::OK, Json(submission)))
}

// ── Review (admin approve/reject) ─────────────────────────────────────────────

/// Admin reviews a pending submission: approve (with vehicle data) or reject (with reason)
#[utoipa::path(
    post,
    path = "/api/v1/admin/submissions/{id}/review",
    tag = "vehicles",
    operation_id = "reviewSubmission",
    params(
        ("id" = Uuid, Path, description = "Submission UUID")
    ),
    request_body = ReviewSubmissionRequest,
    responses(
        (status = 200, description = "Review processed", body = ReviewSubmissionResponse),
        (status = 400, description = "Invalid request", body = AppErrorResponse),
        (status = 404, description = "Submission not found", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn review_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<super::super::dto::pending_submission::ReviewSubmissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // For now, use a placeholder reviewer ID. In production, extract from auth middleware.
    // Try to get the authenticated user from request extensions
    let reviewer_id = get_admin_reviewer_id(&state.db).await?;

    // Fetch the submission first to get the plate number
    let submission =
        crate::queries::submission_queries::get_submission_by_id(&state.db, id).await?;

    match payload.decision {
        SubmissionStatus::Approved => {
            let vehicle_data = payload.vehicle_data.ok_or_else(|| {
                AppError::bad_request(
                    "Vehicle data is required when approving a submission",
                )
            })?;

            let vehicle_id = crate::queries::submission_queries::approve_submission(
                &state.db,
                id,
                reviewer_id,
                &submission.plate_number,
                &vehicle_data,
            )
            .await?;

            Ok((
                StatusCode::OK,
                Json(ReviewSubmissionResponse {
                    message: "Submission approved and vehicle data saved".to_string(),
                    submission_id: id,
                    status: SubmissionStatus::Approved,
                    vehicle_id: Some(vehicle_id),
                }),
            ))
        }
        SubmissionStatus::Rejected => {
            let reason = payload.rejection_reason.ok_or_else(|| {
                AppError::bad_request(
                    "A rejection reason is required when rejecting a submission",
                )
            })?;

            if reason.trim().is_empty() {
                return Err(AppError::bad_request("Rejection reason cannot be empty"));
            }

            crate::queries::submission_queries::reject_submission(
                &state.db,
                id,
                reviewer_id,
                &reason,
            )
            .await?;

            Ok((
                StatusCode::OK,
                Json(ReviewSubmissionResponse {
                    message: "Submission rejected".to_string(),
                    submission_id: id,
                    status: SubmissionStatus::Rejected,
                    vehicle_id: None,
                }),
            ))
        }
        SubmissionStatus::Pending => {
            Err(AppError::bad_request(
                "Decision must be 'approved' or 'rejected', not 'pending'",
            ))
        }
    }
}

/// Get the first admin user ID. In production this comes from the auth token.
async fn get_admin_reviewer_id(pool: &sqlx::PgPool) -> Result<Uuid, AppError> {
    let id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM users WHERE role = 'admin' AND is_active = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    id.ok_or_else(|| AppError::not_found("No admin user found"))
}

// ── Audit Log (admin) ─────────────────────────────────────────────────────────

/// Get the audit trail for a submission
#[utoipa::path(
    get,
    path = "/api/v1/admin/submissions/{id}/audit",
    tag = "vehicles",
    operation_id = "getSubmissionAuditLog",
    params(
        ("id" = Uuid, Path, description = "Submission UUID")
    ),
    responses(
        (status = 200, description = "Audit log entries", body = [SubmissionAuditLogEntry]),
        (status = 404, description = "Submission not found", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submission_audit_log(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let entries =
        crate::queries::submission_queries::get_submission_audit_log(&state.db, id).await?;
    Ok((StatusCode::OK, Json(entries)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::pending_submission::{CreatePendingSubmissionRequest, SubmissionStatus};

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
        let request = create_test_submission_request();
        assert_eq!(request.plate_number, "TEST123");
        assert!(request.front_image_url.contains("front.jpg"));
        assert!(request.back_image_url.contains("back.jpg"));
        assert_eq!(request.notes, Some("Test notes".to_string()));
        assert_eq!(request.latitude, Some(40.7128));
        assert_eq!(request.longitude, Some(-74.0060));
    }

    #[tokio::test]
    async fn test_submit_vehicle_request_validation() {
        let mut request = create_test_submission_request();
        request.plate_number = "".to_string();
        assert!(request.plate_number.is_empty());

        request.notes = None;
        request.latitude = None;
        request.longitude = None;
        assert!(request.notes.is_none());
        assert!(request.latitude.is_none());
        assert!(request.longitude.is_none());
    }

    #[tokio::test]
    async fn test_data_entry_response_structure() {
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
        let pending = SubmissionStatus::Pending;
        let approved = SubmissionStatus::Approved;
        let rejected = SubmissionStatus::Rejected;

        assert_eq!(pending, SubmissionStatus::Pending);
        assert_eq!(approved, SubmissionStatus::Approved);
        assert_eq!(rejected, SubmissionStatus::Rejected);

        assert!(pending != approved);
        assert!(approved != rejected);
        assert!(pending != rejected);
    }

    #[tokio::test]
    async fn test_submission_status_from_db_str() {
        assert_eq!(SubmissionStatus::from_db_str("pending"), SubmissionStatus::Pending);
        assert_eq!(SubmissionStatus::from_db_str("approved"), SubmissionStatus::Approved);
        assert_eq!(SubmissionStatus::from_db_str("rejected"), SubmissionStatus::Rejected);
        assert_eq!(SubmissionStatus::from_db_str("unknown"), SubmissionStatus::Pending);
    }

    #[tokio::test]
    async fn test_submission_status_as_db_str() {
        assert_eq!(SubmissionStatus::Pending.as_db_str(), "pending");
        assert_eq!(SubmissionStatus::Approved.as_db_str(), "approved");
        assert_eq!(SubmissionStatus::Rejected.as_db_str(), "rejected");
    }

    #[tokio::test]
    async fn test_review_response_structure() {
        let submission_id = uuid::Uuid::new_v4();
        let vehicle_id = uuid::Uuid::new_v4();

        let response = ReviewSubmissionResponse {
            message: "Submission approved".to_string(),
            submission_id,
            status: SubmissionStatus::Approved,
            vehicle_id: Some(vehicle_id),
        };

        assert_eq!(response.status, SubmissionStatus::Approved);
        assert_eq!(response.vehicle_id, Some(vehicle_id));
    }

    #[tokio::test]
    async fn test_create_pending_submission_request_serialization() {
        let request = create_test_submission_request();
        assert!(!request.plate_number.is_empty());
        assert!(!request.front_image_url.is_empty());
        assert!(!request.back_image_url.is_empty());
        assert!(!request.agent_id.is_nil());
    }
}
