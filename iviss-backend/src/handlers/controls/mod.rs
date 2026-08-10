use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::instrument;

use crate::{
    app_state::AppState,
    dto::{
        create_control::{CreateControlRequest, CreateControlResponse},
        list_control::{ControlListQuery, ControlPagedQuery, PagedControlsResponse},
    },
    errors::AppError,
};

#[utoipa::path(
    post,
    path = "/api/v1/controls",
    tag = "controls",
    request_body = CreateControlRequest,
    operation_id = "createControl",
    responses(
        (status = 201, description = "Control created", body = CreateControlResponse),
        (status = 400, description = "Invalid request", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(name = "control.create", skip(state, payload))]
pub async fn create_control(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateControlRequest>,
) -> Result<impl IntoResponse, AppError> {
    let id = crate::queries::controls::create_control_record(&state.db, payload).await?;

    let response = CreateControlResponse {
        id,
        message: "Control logged successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/controls",
    tag = "controls",
    params(
        ("start_date" = Option<String>, Query, description = "Filter controls from this date (inclusive)" ),
        ("end_date" = Option<String>, Query, description = "Filter controls until this date (inclusive)"),
        ("agent_id" = Option<Uuid>, Query, description = "Filter controls by agent UUID"),
        ("status" = Option<Status>, Query, description = "Filter controls by status" ),
        ("plate" = Option<String>, Query, description = "Filter controls by plate number" )
     ),
    operation_id = "getControls",
    responses(
        (status = 200, description = "List of control records", body = [ListControlResponse]),
         (status = 400, description = "Invalid request",        body = AppErrorResponse, 
             example = json!({ "code": "INVALID_REQUEST", "message": "Invalid date format for 'start_date'" })),
         (status = 404, description = "Not found",           body = AppErrorResponse, 
             example = json!({ "code": "NOT_FOUND", "message": "No controls found matching the provided filters" })),
         (status = 500, description = "Internal server error",  body = AppErrorResponse, 
             example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]

pub async fn get_list_control(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ControlListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let controls = crate::queries::controls::get_control_records(
        &state.db,
        query.start_date,
        query.end_date,
        query.agent_id,
        query.status,
        query.plate,
    )
    .await?;

    Ok((StatusCode::OK, Json(controls)))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/controls/paged",
    tag = "controls",
    params(ControlPagedQuery),
    operation_id = "getControlsPaged",
    responses(
        (status = 200, description = "Paged control records", body = PagedControlsResponse),
        (status = 400, description = "Invalid request", body = AppErrorResponse),
        (status = 401, description = "Unauthorized", body = AppErrorResponse),
        (status = 500, description = "Internal server error", body = AppErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_list_control_paged(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ControlPagedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);

    let (items, total) =
        crate::queries::controls::get_paged_control_records(&state.db, &query, page, page_size)
            .await?;

    Ok((
        StatusCode::OK,
        Json(PagedControlsResponse {
            items,
            total,
            page,
            page_size,
        }),
    ))
}
