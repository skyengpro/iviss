use crate::app_state::AppState;
use crate::dto::organizations::{CreateOrganizationRequest, UpdateOrganizationRequest};
use crate::errors::AppError;
use crate::queries::organization_queries::{
    create_organization as create_org_query, delete_organization as delete_org_query,
    get_organization_by_id as get_org_query, update_organization as update_org_query,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Create a new organization (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/admin/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = Organization),
        (status = 400, description = "Bad request - validation error", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden - admin only", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "createOrganization",
    security(("bearer_auth" = []))
)]
pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let org = create_org_query(&state.db, payload).await?;

    tracing::info!(
        organization_id = %org.id,
        organization_name = %org.name,
        "Organization created successfully"
    );

    Ok((StatusCode::CREATED, Json(org)))
}

/// Get organization details by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/organizations/{id}",
    responses(
        (status = 200, description = "Organization details retrieved", body = OrganizationDetails),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden - admin only", body = AppErrorResponse),
        (status = 404, description = "Organization not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "getOrganization",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let org = get_org_query(&state.db, id).await?;
    Ok((StatusCode::OK, Json(org)))
}

/// Update an existing organization (admin only)
#[utoipa::path(
    put,
    path = "/api/v1/admin/organizations/{id}",
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated successfully", body = Organization),
        (status = 400, description = "Bad request - validation error", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden - admin only", body = AppErrorResponse),
        (status = 404, description = "Organization not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "updateOrganization",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrganizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let org = update_org_query(&state.db, id, payload).await?;

    tracing::info!(
        organization_id = %org.id,
        organization_name = %org.name,
        "Organization updated successfully"
    );

    Ok((StatusCode::OK, Json(org)))
}

/// Delete an organization (admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/organizations/{id}",
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 400, description = "Bad request - organization has active users", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 403, description = "Forbidden - admin only", body = AppErrorResponse),
        (status = 404, description = "Organization not found", body = AppErrorResponse)
    ),
    tag = "admin",
    operation_id = "deleteOrganization",
    params(
        ("id" = Uuid, Path, description = "Organization ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_organization(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    delete_org_query(&state.db, id).await?;

    tracing::info!(
        organization_id = %id,
        "Organization deleted successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}
