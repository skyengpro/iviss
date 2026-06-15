use crate::app_state::AppState;
use crate::{
    dto::{
        common::{IdentificationMode, Status},
        list_control::ControlResults,
        search_vehicle::{VehicleSearchRequest, VehicleSearchResult},
    },
    errors::AppError,
    services::vehicle_client_service::VehicleApiService,
    utils::plate_format,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

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
#[instrument(name = "vehicle.search", skip(state, payload), fields(plate = %payload.plate))]
pub async fn search_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate plate format
    let plate = validate_plate_format(&payload.plate)?;

    let api_response = state
        .vehicle_api_svc
        .query_plate(&plate)
        .await
        .map_err(|error| {
            tracing::error!("Vehicle API lookup failed for plate {}: {}", plate, error);
            AppError::external_api_failure("Vehicle registry lookup failed")
        })?;

    let vehicle_info = api_response.vehicle;
    let status_results = VehicleApiService::build_status_results_from_api(&vehicle_info);

    // Contextual logging for search location (using the fields to avoid "unused field" warning)
    if let (Some(lat), Some(lon)) = (payload.latitude, payload.longitude) {
        tracing::info!(
            "Vehicle search for plate {} at coordinates: {}, {}",
            plate,
            lat,
            lon
        );
    } else {
        tracing::info!("Vehicle search for plate {} (no location provided)", plate);
    }

    // Auto-log control record for successful search
    let control_id = Uuid::new_v4();
    let current_time = time::OffsetDateTime::now_utc();

    let original_plate = api_response.plate_number.unwrap_or_else(|| plate.clone());
    let status_str = status_to_control_value(&status_results.overall_status);
    let control_results = ControlResults {
        registration: Status::Valid,
        insurance: status_results.insurance.status.clone(),
        technical_inspection: status_results.technical.status.clone(),
        wanted_status: status_results.police.status.clone(),
        customs_status: status_results.customs.status.clone(),
    };

    // Insert control record
    let _ = sqlx::query(
        r#"
        INSERT INTO control_records (
            id, plate_number, agent_id, organization_id, timestamp,
            latitude, longitude, address, identification_mode, ocr_confidence,
            overall_status, results_json, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(control_id)
    .bind(&original_plate)
    .bind(payload.agent_id.unwrap_or_else(Uuid::new_v4))
    .bind(payload.organization_id.unwrap_or_else(Uuid::new_v4))
    .bind(current_time)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(payload.address.clone())
    .bind("manual")
    .bind(1.0)
    .bind(status_str)
    .bind(
        serde_json::to_value(&control_results).unwrap_or_else(|error| {
            tracing::error!("Failed to serialize control results: {}", error);
            serde_json::json!({})
        }),
    )
    .bind("Auto-logged via vehicle search")
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to auto-log control: {}", e);
        e
    });

    // Determine identification mode and confidence (simplified for now)
    let identification_mode = IdentificationMode::Manual;
    let confidence = Some(1.0); // Perfect confidence for manual input

    let response = VehicleSearchResult {
        plate_number: original_plate,
        confidence,
        identification_mode: Some(identification_mode),
        vehicle: vehicle_info,
        status_results,
    };

    Ok((StatusCode::OK, Json(response)))
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
#[instrument(name = "vehicle.search_v1", skip(state, payload), fields(plate = %payload.plate))]
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
