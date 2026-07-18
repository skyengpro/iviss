//! `POST /query` — single-plate lookup.
//!
//! Mirrors the interface of the real external vehicle registry API exactly:
//! - Accepts a plain-text body containing the plate number (any case / spacing).
//! - Returns `{"data": "<html>..."}` on success.
//! - Returns `{"data": "... Service indisponible ..."}` when not found.
//! - Returns 401 when Basic Auth is missing or incorrect.

use axum::{
    body::Bytes,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{auth::ValidatedBasicAuth, db, html_builder, AppState};

/// Handler for `POST /query`.
pub async fn query_plate(
    State(state): State<AppState>,
    _auth: ValidatedBasicAuth,
    body: Bytes,
) -> Response {
    let plate = match std::str::from_utf8(&body) {
        Ok(s) => s.trim().to_string(),
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid UTF-8 in request body").into_response()
        }
    };

    if plate.is_empty() {
        return (StatusCode::BAD_REQUEST, "Plate number required in request body").into_response();
    }

    tracing::debug!(plate = %plate, "Plate lookup requested");

    match db::find_by_plate(&state.pool, &plate).await {
        Ok(Some(vehicle)) => {
            let body = html_builder::build_found_response(&vehicle);
            tracing::debug!(plate = %plate, customs_status = ?vehicle.customs_status, "Plate found");
            json_response(body)
        }
        Ok(None) => {
            tracing::debug!(plate = %plate, "Plate not found");
            let body = html_builder::build_not_found_response(&plate);
            json_response(body)
        }
        Err(e) => {
            tracing::error!(error = %e, "DB error during plate lookup");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn json_response(body: String) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}
