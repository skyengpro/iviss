use crate::app_state::AppState;
use crate::dto::{common, pending_submission};
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::rbac::AuthenticatedAdmin;
use crate::services::vehicles::data_cache::UnregisteredScope;
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

// ── Submit (agent-facing) ─────────────────────────────────────────────────────

#[allow(unused_imports)]
use crate::dto::pending_submission::DataEntryResponse;
use crate::dto::pending_submission::SubmissionListQuery;

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
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<pending_submission::CreatePendingSubmissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let location = common::SubmissionLocation {
        latitude: payload.latitude,
        longitude: payload.longitude,
        address: None,
    };
    let submission_id = crate::queries::submissions::create_pending_submission(
        &state.db,
        user.user_id,
        payload.plate_number.clone(),
        payload.front_image_url,
        payload.back_image_url,
        payload.notes,
        location,
    )
    .await?;

    let response = pending_submission::DataEntryResponse {
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
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<pending_submission::CreatePendingSubmissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    submit_vehicle(State(state), Extension(user), Json(payload)).await
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
    Extension(admin): Extension<AuthenticatedAdmin>,
    Query(query): Query<SubmissionListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org_scope = admin_org_scope(&admin)?;

    let mut submissions = crate::queries::submissions::get_pending_submissions(
        &state.db,
        org_scope,
        query.status.as_deref(),
    )
    .await?;

    let include_unregistered = query
        .status
        .as_deref()
        .is_none_or(|status| status.eq_ignore_ascii_case("pending"));

    if include_unregistered {
        if let Some(s3_data_cache) = &state.s3_data_cache {
            let s3_scope = match org_scope {
                None => UnregisteredScope::AllTenants,
                Some(id) => UnregisteredScope::Organization(id),
            };
            match s3_data_cache.list_unregistered(s3_scope).await {
                Ok(unregistered) => {
                    submissions.extend(unregistered.into_iter().map(unregistered_to_list_item));
                }
                Err(error) => {
                    tracing::warn!(error = %error, "failed to list unregistered plates from S3; serving submissions from DB only");
                }
            }
        }
    }

    submissions.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));

    Ok((StatusCode::OK, Json(submissions)))
}

fn unregistered_to_list_item(
    plate: crate::services::vehicles::data_cache::UnregisteredPlate,
) -> pending_submission::PendingSubmissionListItem {
    let submitted_at = plate
        .marked_at
        .and_then(|dt| {
            dt.format(&time::format_description::well_known::Rfc3339)
                .ok()
        })
        .unwrap_or_default();

    pending_submission::PendingSubmissionListItem {
        id: None,
        plate_number: plate.plate_number,
        agent_name: None,
        status: pending_submission::SubmissionStatus::Pending,
        submitted_at,
        source: pending_submission::SubmissionSource::S3Unregistered,
    }
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
    Extension(admin): Extension<AuthenticatedAdmin>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let org_scope = admin_org_scope(&admin)?;
    let submission: pending_submission::PendingSubmissionDetail =
        crate::queries::submissions::get_submission_by_id(&state.db, id, org_scope).await?;
    Ok((StatusCode::OK, Json(submission)))
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
    Extension(admin): Extension<AuthenticatedAdmin>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let org_scope = admin_org_scope(&admin)?;
    let entries =
        crate::queries::submissions::get_submission_audit_log(&state.db, id, org_scope).await?;
    Ok((StatusCode::OK, Json(entries)))
}

fn admin_org_scope(admin: &AuthenticatedAdmin) -> Result<Option<Uuid>, AppError> {
    match admin.role.as_str() {
        "admin" => Ok(None),
        _ => admin
            .organization_id
            .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))
            .map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::pending_submission::{
        CreatePendingSubmissionRequest, ReviewSubmissionResponse, SubmissionStatus,
    };

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
        let response = pending_submission::DataEntryResponse {
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
        assert_eq!(
            SubmissionStatus::from_db_str("pending"),
            SubmissionStatus::Pending
        );
        assert_eq!(
            SubmissionStatus::from_db_str("approved"),
            SubmissionStatus::Approved
        );
        assert_eq!(
            SubmissionStatus::from_db_str("rejected"),
            SubmissionStatus::Rejected
        );
        assert_eq!(
            SubmissionStatus::from_db_str("unknown"),
            SubmissionStatus::Pending
        );
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
