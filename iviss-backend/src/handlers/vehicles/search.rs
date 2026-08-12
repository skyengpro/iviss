use crate::app_state::AppState;
use crate::{
    dto::{
        common::{IdentificationMode, Status},
        controls::ControlResults,
        search_vehicle::{VehicleDataSource, VehicleSearchRequest, VehicleSearchResult},
    },
    errors::AppError,
    external_services::vehicle_client::{VehicleApiError, VehicleApiResponse},
    services::vehicles::status::VehicleService,
    utils::plate_format,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

/// Maximum time to wait for an S3 cache read when the external API is down.
const S3_CACHE_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

// ── POST /api/v1/vehicles/search ──────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/vehicles/search",
    tag = "vehicles",
    operation_id = "searchVehicle",
    request_body = VehicleSearchRequest,
    responses(
        (status = 200, description = "Vehicle found with status results", body = VehicleSearchResult),
        (status = 400, description = "Invalid plate format",              body = AppErrorResponse, 
             example = json!({ "code": "INVALID_PLATE", "message": "Plate number must be 6-8 alphanumeric characters" })),
         (status = 401, description = "Unauthorized",                      body = AppErrorResponse, 
             example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
         (status = 404, description = "Plate not found in registry",       body = AppErrorResponse,     
             example = json!({ "code": "NOT_FOUND", "message": "No vehicle found with the provided plate  number" })),
         (status = 500, description = "Internal server error",             body = AppErrorResponse, 
             example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
     ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(name = "vehicle.search", skip(state, payload))]
pub async fn search_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate plate format
    let plate = validate_plate_format(&payload.plate)?;

    log_search_location(&payload);

    if !state.vehicle_api_enabled {
        tracing::warn!("Vehicle API disabled via ENABLE_VEHICLE_API; using cache fallback");
        return handle_vehicle_api_unavailable(&state, &plate, &payload).await;
    }

    match state.vehicle_api_svc.query_plate(&plate).await {
        Ok(api_response) => {
            let response = build_search_result(api_response, &plate);

            spawn_write_through(&state, &plate, &response.vehicle);
            record_vehicle_search_control(&state, &payload, &response).await;

            Ok((StatusCode::OK, Json(response)))
        }
        Err(VehicleApiError::NotFound) => {
            tracing::info!(plate = %plate, "Vehicle not found in external registry");

            Err(AppError::not_found(
                "No vehicle found with the provided plate number",
            ))
        }
        Err(error) => {
            tracing::error!("Vehicle API lookup failed: {}", error);
            handle_vehicle_api_unavailable(&state, &plate, &payload).await
        }
    }
}

/// Shared fallback when the vehicle API cannot be reached — either because it
/// failed, or because it was disabled via `ENABLE_VEHICLE_API`. Tries the S3
/// cache, enqueueing a retry marker on miss.
async fn handle_vehicle_api_unavailable(
    state: &AppState,
    plate: &str,
    payload: &VehicleSearchRequest,
) -> Result<(StatusCode, Json<VehicleSearchResult>), AppError> {
    if let Some(s3_data_cache) = &state.s3_data_cache {
        match tokio::time::timeout(S3_CACHE_READ_TIMEOUT, s3_data_cache.get_vehicle_data(plate))
            .await
        {
            Ok(Ok(Some(cached))) => {
                tracing::info!(
                    cached_at = %cached.cached_at,
                    "Serving vehicle data from S3 cache"
                );
                let status_results = VehicleService::build_status_results_from_api(&cached.vehicle);
                let cached_at = cached
                    .cached_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .ok();
                let response = VehicleSearchResult {
                    plate_number: cached.plate_number,
                    confidence: Some(1.0),
                    identification_mode: Some(IdentificationMode::Manual),
                    vehicle: cached.vehicle,
                    status_results,
                    source: Some(VehicleDataSource::Cache),
                    cached_at,
                };
                record_vehicle_search_control(state, payload, &response).await;
                return Ok((StatusCode::OK, Json(response)));
            }
            Ok(Ok(None)) => {
                tracing::warn!("Vehicle S3 cache miss");
            }
            Ok(Err(cache_error)) => {
                tracing::warn!("Vehicle S3 cache lookup failed: {}", cache_error);
            }
            Err(_) => {
                tracing::warn!("Vehicle S3 cache read timed out");
            }
        }

        spawn_enqueue_retry(state, plate);
    }

    Err(AppError::external_api_failure(
        "Vehicle registry lookup failed",
    ))
}

/// Detached: never blocks or degrades the agent's response.
fn spawn_write_through(
    state: &AppState,
    plate: &str,
    vehicle: &crate::dto::search_vehicle::VehicleInfo,
) {
    if let Some(cache) = &state.s3_data_cache {
        let (cache, plate, vehicle) = (cache.clone(), plate.to_string(), vehicle.clone());
        tokio::spawn(async move {
            if let Err(e) = cache.store_vehicle_data(&plate, &vehicle).await {
                tracing::warn!(error = %e, "S3 write-through failed");
            }
        });
    }
}

fn spawn_enqueue_retry(state: &AppState, plate: &str) {
    if let Some(cache) = &state.s3_data_cache {
        let (cache, plate) = (cache.clone(), plate.to_string());
        tokio::spawn(async move {
            if let Err(e) = cache.enqueue_retry(&plate).await {
                tracing::warn!(error = %e, "failed to enqueue S3 retry marker");
            }
        });
    }
}

fn build_search_result(
    api_response: VehicleApiResponse,
    requested_plate: &str,
) -> VehicleSearchResult {
    let vehicle_info = api_response.vehicle;
    let status_results = VehicleService::build_status_results_from_api(&vehicle_info);
    let original_plate = api_response
        .plate_number
        .unwrap_or_else(|| requested_plate.to_string());

    VehicleSearchResult {
        plate_number: original_plate,
        confidence: Some(1.0),
        identification_mode: Some(IdentificationMode::Manual),
        vehicle: vehicle_info,
        status_results,
        source: Some(VehicleDataSource::Live),
        cached_at: None,
    }
}

fn log_search_location(payload: &VehicleSearchRequest) {
    if let (Some(lat), Some(lon)) = (payload.latitude, payload.longitude) {
        tracing::info!("Vehicle search at coordinates: {}, {}", lat, lon);
    } else {
        tracing::info!("Vehicle search (no location provided)");
    }
}

async fn record_vehicle_search_control(
    state: &AppState,
    payload: &VehicleSearchRequest,
    response: &VehicleSearchResult,
) {
    let control_id = Uuid::new_v4();
    let current_time = time::OffsetDateTime::now_utc();
    let status_str = status_to_control_value(&response.status_results.overall_status);
    let control_results = ControlResults {
        registration: Status::Valid,
        insurance: response.status_results.insurance.status.clone(),
        technical_inspection: response.status_results.technical.status.clone(),
        wanted_status: response.status_results.police.status.clone(),
        customs_status: response.status_results.customs.status.clone(),
    };

    let results_json = serde_json::to_value(&control_results).unwrap_or_else(|error| {
        tracing::error!("Failed to serialize control results: {}", error);
        serde_json::json!({})
    });

    let _ = crate::queries::vehicles::insert_control_record_for_vehicle_search(
        &state.db,
        crate::queries::vehicles::VehicleSearchControlRecordInsert {
            control_id,
            plate_number: &response.plate_number,
            agent_id: payload.agent_id.unwrap_or_else(Uuid::new_v4),
            organization_id: payload.organization_id.unwrap_or_else(Uuid::new_v4),
            timestamp: current_time,
            latitude: payload.latitude,
            longitude: payload.longitude,
            address: payload.address.clone(),
            overall_status: status_str,
            results_json,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to auto-log control: {}", e);
        e
    });
}

#[utoipa::path(
    post,
    path = "/api/v1/vehicles/search",
    tag = "vehicles",
    operation_id = "searchVehicleV1",
    request_body = VehicleSearchRequest,
    responses(
        (status = 200, description = "Vehicle found with status results", body = VehicleSearchResult),
        (status = 400, description = "Invalid plate format",              body = AppErrorResponse,
             example = json!({ "code": "INVALID_PLATE", "message": "Plate number must be 6-8 alphanumeric characters" })),
         (status = 401, description = "Unauthorized",                      body = AppErrorResponse,
             example = json!({ "code": "UNAUTHORIZED", "message": "Invalid token" })),
         (status = 404, description = "Plate not found in registry",       body = AppErrorResponse,
             example = json!({ "code": "NOT_FOUND", "message": "No vehicle found with the provided plate  number" })),
         (status = 500, description = "Internal server error",             body = AppErrorResponse,
             example = json!({ "code": "INTERNAL_ERROR", "message": "Internal Server Error" })),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(name = "vehicle.search_v1", skip(state, payload))]
pub async fn search_vehicle_v1(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    search_vehicle(State(state), Json(payload)).await
}

pub fn validate_plate_format(plate: &str) -> Result<String, AppError> {
    let trimmed = plate.trim();

    if trimmed.is_empty() {
        return Err(AppError::bad_request("Plate number cannot be empty"));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || c == '-')
    {
        return Err(AppError::bad_request(format!(
            "Invalid plate format: '{plate}'. Only letters, digits, spaces and dashes are allowed"
        )));
    }

    let compact = plate_format::normalise(trimmed);
    if !plate_format::is_valid(&compact) {
        return Err(AppError::bad_request(format!(
            "Invalid plate format: '{plate}'. Supported formats include CE 128 BC, LT 3334 W, LT SR 9652 A, AN 9652 E, PA 02 RC 521, IT 21052 RC, CE 2456 WG, WT 1202082, PT 01200, IS 245642 RC, SN 1234, 1234567 and RT123456"
        )));
    }

    Ok(compact)
}

fn status_to_control_value(status: &crate::dto::common::Status) -> &'static str {
    match status {
        crate::dto::common::Status::Valid => "valid",
        crate::dto::common::Status::Warning => "warning",
        crate::dto::common::Status::Critical => "critical",
        crate::dto::common::Status::Pending => "pending",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_plate_format tests ─────────────────────────────────────────

    #[test]
    fn test_validate_plate_format_valid_standard() {
        let result = validate_plate_format("CE128BC");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CE128BC");
    }

    #[test]
    fn test_validate_plate_format_valid_with_spaces() {
        let result = validate_plate_format("CE 128 BC");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CE128BC");
    }

    #[test]
    fn test_validate_plate_format_valid_with_dashes() {
        let result = validate_plate_format("CE-128-BC");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CE128BC");
    }

    #[test]
    fn test_validate_plate_format_valid_lowercase() {
        let result = validate_plate_format("ce128bc");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CE128BC");
    }

    #[test]
    fn test_validate_plate_format_valid_with_whitespace() {
        let result = validate_plate_format("  CE128BC  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CE128BC");
    }

    #[test]
    fn test_validate_plate_format_standard_long() {
        // Standard long: REGION + 4 digits + 1 letter
        let result = validate_plate_format("LT 3334 W");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "LT3334W");
    }

    #[test]
    fn test_validate_plate_format_police() {
        // Police/Security: SN + 4 digits
        let result = validate_plate_format("SN 1234");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "SN1234");
    }

    #[test]
    fn test_validate_plate_format_military() {
        // Military: 7 digits
        let result = validate_plate_format("1234567");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1234567");
    }

    #[test]
    fn test_validate_plate_format_state() {
        // State/Govt: 2 letters + 4 digits + 1 letter
        let result = validate_plate_format("EN1234X");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "EN1234X");
    }

    #[test]
    fn test_validate_plate_format_postal() {
        // Postal: RT + 6 digits
        let result = validate_plate_format("RT123456");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "RT123456");
    }

    #[test]
    fn test_validate_plate_format_diplomatic() {
        // Diplomatic: CD + digits + digits
        let result = validate_plate_format("CD 34 444");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "CD34444");
    }

    #[test]
    fn test_validate_plate_format_invalid_too_short() {
        let result = validate_plate_format("CE128B");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_invalid_too_long() {
        let result = validate_plate_format("CE128BCDE");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_invalid_wrong_pattern() {
        let result = validate_plate_format("CE1ABBC");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_invalid_all_letters() {
        let result = validate_plate_format("ABCDEFG");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_invalid_empty() {
        let result = validate_plate_format("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_invalid_special_chars() {
        let result = validate_plate_format("CE@128BC");
        assert!(result.is_err());
    }
}
