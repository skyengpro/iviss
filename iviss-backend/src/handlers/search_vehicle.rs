use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};

use crate::{
    dto::{
        common::IdentificationMode,
        search_vehicle::{VehicleSearchRequest, VehicleSearchResult},
    },
    errors::{AppError, AppErrorResponse},
    // middleware::auth::AuthClaims,
};

// ── GET /vehicles/{plate_number} ──────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/vehicles/search",
    tag = "vehicles",
    request_body = VehicleSearchRequest,
    responses(
        (status = 200, description = "Vehicle found with status results", body = VehicleSearchResult),
        (status = 400, description = "Invalid plate format",              body = AppErrorResponse, 
            example = json!({ "code": "INVALID_PLATE", "message": "Plate number must be 6-8 alphanumeric characters" })),
        (status = 401, description = "Unauthorized",                      body = AppErrorResponse, 
            example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
        (status = 404, description = "Plate not found in registry",       body = AppErrorResponse, 
        
            example = json!({ "code": "NOT_FOUND", "message": "No vehicle found with the provided plate  number" })),
        (status = 500, description = "Internal server error",             body = AppErrorResponse, example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]

pub async fn search_vehicle(_payload: Json<VehicleSearchRequest>) -> impl IntoResponse {
    // TODO:
    StatusCode::ACCEPTED
}
