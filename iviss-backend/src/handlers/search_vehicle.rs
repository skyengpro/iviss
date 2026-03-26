use crate::app_state::AppState;
use crate::{
    dto::{
        common::IdentificationMode,
        search_vehicle::{VehicleSearchRequest, VehicleSearchResult},
    },
    errors::AppError,
    queries::vehicle_queries::{get_vehicle_status_by_plate, get_vehicle_with_owner_by_plate},
    services::vehicle_service::VehicleService,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use uuid::Uuid;

// ── GET /vehicles/{plate_number} ──────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/vehicles/search",
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

pub async fn search_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate plate format
    let plate = validate_plate_format(&payload.plate)?;

    // Query vehicle data from database
    let vehicle_row: crate::models::search_vehicle::VehicleRow =
        get_vehicle_with_owner_by_plate(&state.db, &plate)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!("No vehicle found with plate number: {}", plate))
            })?;

    // Query vehicle status data
    let status_row: Option<crate::queries::vehicle_queries::VehicleStatusRow> =
        get_vehicle_status_by_plate(&state.db, &plate).await?;

    // Build response using service layer
    let vehicle_info = VehicleService::build_vehicle_info(&vehicle_row);
    let status_results = VehicleService::build_status_results(&status_row);

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

    // Use original plate_number with spaces from database for display
    let original_plate = &vehicle_row.plate_number;

    let status_str = if status_row.as_ref().is_some_and(|s| s.stolen_status) {
        "critical"
    } else if status_row.as_ref().is_some_and(|s| {
        s.insurance_status.as_ref().is_some_and(|i| i == "expired")
            || s.technical_status.as_ref().is_some_and(|t| t == "expired")
    }) {
        "warning"
    } else {
        "valid"
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
    .bind(original_plate)
    .bind(payload.agent_id.unwrap_or_else(Uuid::new_v4))
    .bind(payload.organization_id.unwrap_or_else(Uuid::new_v4))
    .bind(current_time)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(payload.address.clone())
    .bind("manual")
    .bind(1.0)
    .bind(status_str)
    .bind(serde_json::json!({
        "insurance": status_row.as_ref().and_then(|s| s.insurance_status.clone()),
        "technical": status_row.as_ref().and_then(|s| s.technical_status.clone()),
        "stolen": status_row.as_ref().map(|s| s.stolen_status),
    }))
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
        plate_number: vehicle_row.plate_number,
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
pub async fn search_vehicle_v1(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VehicleSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    search_vehicle(State(state), Json(payload)).await
}

pub fn validate_plate_format(plate: &str) -> Result<String, AppError> {
    // Normalize: remove all whitespace/dashes for internal lookup
    let normalized = plate.trim().to_uppercase().replace([' ', '-'], "");

    // Validates 7-character Cameroon format (LLDDDLL)
    static COMPACT_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[A-Z]{2}\d{3}[A-Z]{2}$").unwrap());

    if !COMPACT_REGEX.is_match(&normalized) {
        return Err(AppError::bad_request(format!(
            "Invalid plate format: '{plate}'. Expected 7-character Cameroon format (e.g. CE128BC)"
        )));
    }
    Ok(normalized)
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
    fn test_validate_plate_format_invalid_too_short() {
        let result = validate_plate_format("CE128B");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_too_long() {
        let result = validate_plate_format("CE128BCD");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_wrong_pattern() {
        // Missing digit in middle
        let result = validate_plate_format("CE1ABBC");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_all_digits() {
        let result = validate_plate_format("1234567");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_all_letters() {
        let result = validate_plate_format("ABCDEFG");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_empty() {
        let result = validate_plate_format("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_special_chars() {
        let result = validate_plate_format("CE@128BC");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid plate format"));
    }
}
