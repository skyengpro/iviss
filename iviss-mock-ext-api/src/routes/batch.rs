//! `GET /batch?prefix=CE` — prefix-based bulk vehicle fetch.
//!
//! Returns a JSON array of all vehicles whose plate number starts with the
//! given prefix (case-insensitive, spaces ignored). Intended for use by the
//! `s3-cache-sync` service, which iterates over known plate prefixes at
//! schedule time rather than querying a single plate at a time.
//!
//! Example:
//!   GET /batch?prefix=CE
//!   → [{"plate_number":"CE 568 LR","chassis_number":"...","mark_and_type":"...",...}, ...]

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{auth::ValidatedBasicAuth, db, AppState};

#[derive(Debug, Deserialize)]
pub struct BatchParams {
    /// Plate prefix to match, e.g. "CE", "LT", "SN".
    prefix: String,
}

/// Handler for `GET /batch`.
pub async fn batch_by_prefix(
    State(state): State<AppState>,
    _auth: ValidatedBasicAuth,
    Query(params): Query<BatchParams>,
) -> impl IntoResponse {
    let prefix = params.prefix.trim().to_string();

    if prefix.is_empty() {
        return (StatusCode::BAD_REQUEST, "Query parameter `prefix` is required").into_response();
    }

    // Guard against wildcard abuse — prefixes should be 1–4 alpha chars.
    if prefix.len() > 4 || !prefix.chars().all(|c| c.is_ascii_alphabetic()) {
        return (
            StatusCode::BAD_REQUEST,
            "`prefix` must be 1–4 ASCII letters (e.g. CE, LT, SN)",
        )
            .into_response();
    }

    tracing::debug!(prefix = %prefix, "Batch query requested");

    match db::find_by_prefix(&state.pool, &prefix).await {
        Ok(vehicles) => {
            tracing::debug!(
                prefix = %prefix,
                count = vehicles.len(),
                "Batch query complete"
            );
            Json(vehicles).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "DB error during batch query");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
