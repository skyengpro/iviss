use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::{app_state::AppState, dto::list_control::ControlListQuery, errors::AppError};

#[utoipa::path(
    get,
    path = "/controls",
    tag = "controls",
    params(
        ("start_date" = Option<String>, Path, description = "Filter controls from this date (inclusive)" ),
        ("end_date" = Option<String>, Path, description = "Filter controls until this date (inclusive)"),
        ("agent_id" = Option<Uuid>, Path, description = "Filter controls by agent UUID"),
        ("status" = Option<Status>, Path, description = "Filter controls by status" ),
        ("plate" = Option<String>, Path, description = "Filter controls by plate number" )
     ),
    responses(
        (status = 200, description = "List of control records", body = ListControlResponse),
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
    let controls = crate::queries::control_queries::get_control_records(
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
