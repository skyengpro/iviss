use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    dto::list_control::{ListControlRequest, ListControlResponse},
    errors::AppError,
};

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
        (status = 400, description = "Invalid request",        body = AppError, 
            example = json!({ "code": "INVALID_REQUEST", "message": "Invalid date format for 'start_date'" })),
        (status = 404, description = "Not found",           body = AppError, 
            example = json!({ "code": "NOT_FOUND", "message": "No controls found matching the provided filters" })),
        (status = 500, description = "Internal server error",  body = AppError, 
            example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]

pub async fn get_list_control(_payload: Json<ListControlRequest>) -> impl IntoResponse {
    // TODO:
    StatusCode::ACCEPTED
}
