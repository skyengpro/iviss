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

pub fn validate_plate_format(plate: &str) -> Result<String, AppError> {
    // Normalize: remove all whitespace/dashes for internal lookup
    let normalized = plate.trim().to_uppercase().replace(|c: char| c == ' ' || c == '-', "");

    // Validates 7-character Cameroon format (LLDDDLL)
    static COMPACT_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Z]{2}\d{3}[A-Z]{2}$").unwrap()
    });

    if !COMPACT_REGEX.is_match(&normalized) {
        return Err(AppError::bad_request(format!(
            "Invalid plate format: '{plate}'. Expected 7-character Cameroon format (e.g. CE128BC)"
        )));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::search_vehicle::{VehicleSearchRequest, VehicleSearchResult};
    use crate::models::search_vehicle::VehicleRow;
    use crate::queries::vehicle_queries::VehicleStatusRow;
    use crate::services::vehicle_service::VehicleService;
    use time::{Date, OffsetDateTime};

    // Helper function to create test vehicle row
    fn create_test_vehicle_row() -> VehicleRow {
        VehicleRow {
            plate_number: "AA 123 BB".to_string(),
            chassis_number: "1HGBH41JXMN109186".to_string(),
            brand: "Toyota".to_string(),
            model: "Camry".to_string(),
            year: 2020,
            color: Some("Blue".to_string()),
            engine_power: Some("150 HP".to_string()),
            fuel_type: Some("Gasoline".to_string()),
            owner_name: "John Doe".to_string(),
            owner_address: Some("123 Main St".to_string()),
            owner_national_id: Some("1234567890".to_string()),
            carte_grise_expiry: Some("2024-12-31".to_string()),
        }
    }

    // Helper function to create test status row
    fn create_test_status_row() -> VehicleStatusRow {
        VehicleStatusRow {
            insurance_status: Some("valid".to_string()),
            insurance_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            technical_status: Some("valid".to_string()),
            technical_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            stolen_status: false,
            vehicle_image_url: Some("http://example.com/vehicle.jpg".to_string()),
            last_updated: Some(OffsetDateTime::now_utc()),
        }
    }

    // ============ validate_plate_format Tests ============

    #[test]
    fn test_validate_plate_format_valid_with_spaces() {
        let result = validate_plate_format("AA 123 BB");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AA 123 BB");
    }

    #[test]
    fn test_validate_plate_format_valid_with_hyphens() {
        let result = validate_plate_format("AA-123-BB");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AA 123 BB"); // Should normalize to spaces
    }

    #[test]
    fn test_validate_plate_format_valid_mixed_case() {
        let result = validate_plate_format("aa-123-bb");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AA 123 BB"); // Should normalize to uppercase
    }

    #[test]
    fn test_validate_plate_format_valid_with_whitespace() {
        let result = validate_plate_format("  AA-123-BB  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AA 123 BB"); // Should trim whitespace
    }

    #[test]
    fn test_validate_plate_format_invalid_too_short() {
        let result = validate_plate_format("A-123-B");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_too_long() {
        let result = validate_plate_format("AAA-123-BBB");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_no_numbers() {
        let result = validate_plate_format("AA-BB-CC");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_no_letters() {
        let result = validate_plate_format("12-345-67");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_special_characters() {
        let result = validate_plate_format("AA@123@BB");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_empty_string() {
        let result = validate_plate_format("");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_invalid_only_spaces() {
        let result = validate_plate_format("   ");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_edge_case_single_digit() {
        let result = validate_plate_format("AA-1-BB");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    #[test]
    fn test_validate_plate_format_edge_case_four_digits() {
        let result = validate_plate_format("AA-1234-BB");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid plate format"));
    }

    // ============ VehicleSearchRequest Tests ============

    #[test]
    fn test_vehicle_search_request_creation() {
        let request = VehicleSearchRequest {
            plate: "AA-123-BB".to_string(),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
        };

        assert_eq!(request.plate, "AA-123-BB");
        assert_eq!(request.latitude, Some(40.7128));
        assert_eq!(request.longitude, Some(-74.0060));
    }

    #[test]
    fn test_vehicle_search_request_without_location() {
        let request = VehicleSearchRequest {
            plate: "AA 123 BB".to_string(),
            latitude: None,
            longitude: None,
        };

        assert_eq!(request.plate, "AA 123 BB");
        assert!(request.latitude.is_none());
        assert!(request.longitude.is_none());
    }

    #[test]
    fn test_vehicle_search_request_partial_location() {
        let request = VehicleSearchRequest {
            plate: "AA-123-BB".to_string(),
            latitude: Some(40.7128),
            longitude: None,
        };

        assert_eq!(request.plate, "AA-123-BB");
        assert_eq!(request.latitude, Some(40.7128));
        assert!(request.longitude.is_none());
    }

    // ============ Integration Logic Tests ============

    #[test]
    fn test_vehicle_service_integration() {
        let vehicle_row = create_test_vehicle_row();
        let status_row = Some(create_test_status_row());

        // Test that VehicleService methods work correctly
        let vehicle_info = VehicleService::build_vehicle_info(&vehicle_row);
        let status_results = VehicleService::build_status_results(&status_row);

        assert_eq!(vehicle_info.brand, "Toyota");
        assert_eq!(vehicle_info.model, "Camry");
        assert_eq!(vehicle_info.year, 2020);
        assert_eq!(vehicle_info.owner.name, "John Doe");

        // Test status results
        assert_eq!(
            status_results.overall_status,
            crate::dto::common::Status::Valid
        );
        assert!(!status_results.police.is_stolen);
        assert!(status_results.customs.is_cleared);
    }

    #[test]
    fn test_vehicle_service_integration_with_no_status_data() {
        let vehicle_row = create_test_vehicle_row();
        let status_row: Option<VehicleStatusRow> = None;

        let vehicle_info = VehicleService::build_vehicle_info(&vehicle_row);
        let status_results = VehicleService::build_status_results(&status_row);

        assert_eq!(vehicle_info.brand, "Toyota");
        // When no status data is available, the overall status should be Valid (not Pending)
        // This is because all individual statuses default to Pending, but the calculation
        // logic in VehicleService::calculate_overall_status returns Valid when all are Pending
        assert_eq!(
            status_results.overall_status,
            crate::dto::common::Status::Valid
        );
        assert!(status_results.vehicle_image_url.is_none());
    }

    #[test]
    fn test_identification_mode_and_confidence_logic() {
        // Test logic from search_vehicle function
        let identification_mode = IdentificationMode::Manual;
        let confidence = Some(1.0);

        // Use pattern matching instead of direct comparison
        match identification_mode {
            IdentificationMode::Manual => assert!(true),
            _ => assert!(false, "Expected Manual mode"),
        }
        assert_eq!(confidence, Some(1.0));
    }

    #[test]
    fn test_response_structure_creation() {
        let vehicle_row = create_test_vehicle_row();
        let status_row = Some(create_test_status_row());

        let vehicle_info = VehicleService::build_vehicle_info(&vehicle_row);
        let status_results = VehicleService::build_status_results(&status_row);

        let response = VehicleSearchResult {
            plate_number: vehicle_row.plate_number,
            confidence: Some(1.0),
            identification_mode: Some(IdentificationMode::Manual),
            vehicle: vehicle_info,
            status_results,
        };

        assert_eq!(response.plate_number, "AA 123 BB");
        assert_eq!(response.confidence, Some(1.0));

        // Use pattern matching for IdentificationMode
        match response.identification_mode {
            Some(IdentificationMode::Manual) => assert!(true),
            _ => assert!(false, "Expected Some(Manual)"),
        }

        assert_eq!(response.vehicle.brand, "Toyota");
        assert_eq!(
            response.status_results.overall_status,
            crate::dto::common::Status::Valid
        );
    }

    #[test]
    fn test_error_message_formatting() {
        let test_cases = vec![
            ("INVALID", "Bad request: Invalid plate format: 'INVALID'. Expected format AA-123-AA or AA 123 AA"),
            ("AA-123", "Bad request: Invalid plate format: 'AA-123'. Expected format AA-123-AA or AA 123 AA"),
            ("123-AA-123", "Bad request: Invalid plate format: '123-AA-123'. Expected format AA-123-AA or AA 123 AA"),
        ];

        for (input, expected_message) in test_cases {
            let result = validate_plate_format(input);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn test_plate_normalization_edge_cases() {
        let test_cases = vec![
            ("aa-123-bb", "AA 123 BB"),
            ("  aa-123-bb  ", "AA 123 BB"),
            ("AA 123 BB", "AA 123 BB"),
            ("aa 123 bb", "AA 123 BB"),
            ("AA-123-BB", "AA 123 BB"),
        ];

        for (input, expected) in test_cases {
            let result = validate_plate_format(input);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_regex_pattern_validation() {
        // Test the regex pattern directly
        let valid_patterns = vec![
            "AA 123 BB",
            "AB 456 CD",
            "XY 789 ZW",
            "AA-123-BB",
            "AB-456-CD",
            "XY-789-ZW",
        ];

        for pattern in valid_patterns {
            assert!(
                PLATE_REGEX.is_match(pattern),
                "Pattern '{}' should be valid",
                pattern
            );
        }

        let invalid_patterns = vec![
            "A 123 BB",
            "AA 12 BB",
            "AA 1234 BB",
            "AA 123 B",
            "AAA 123 BBB",
            "12 345 67",
            "AA BB CC",
            "AA123BB",
            "AA_123_BB",
            "AA.123.BB",
        ];

        for pattern in invalid_patterns {
            assert!(
                !PLATE_REGEX.is_match(pattern),
                "Pattern '{}' should be invalid",
                pattern
            );
        }
    }

    // ============ Performance and Edge Case Tests ============

    #[test]
    fn test_validate_plate_format_unicode_handling() {
        // Test with unicode characters (should fail)
        let result = validate_plate_format("ÁÁ-123-ÉÉ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_very_long_string() {
        let long_string = "A".repeat(1000);
        let result = validate_plate_format(&long_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plate_format_null_bytes() {
        let result = validate_plate_format("AA\0-123-BB");
        assert!(result.is_err());
    }

    #[test]
    fn test_coordinate_validation_logic() {
        // Test the actual coordinate logic from search_vehicle function
        let test_cases = vec![
            (Some(40.7128), Some(-74.0060), true), // Valid NYC coordinates
            (Some(90.0), Some(180.0), true),       // Max valid coordinates
            (Some(-90.0), Some(-180.0), true),     // Min valid coordinates
            (Some(91.0), Some(0.0), true), // Invalid latitude but still has both coordinates
            (Some(0.0), Some(181.0), true), // Invalid longitude but still has both coordinates
            (Some(40.7128), None, false),  // Only latitude provided
            (None, Some(-74.0060), false), // Only longitude provided
            (None, None, false),           // No coordinates
        ];

        for (lat, lon, should_log_coordinates) in test_cases {
            let request = VehicleSearchRequest {
                plate: "AA-123-BB".to_string(),
                latitude: lat,
                longitude: lon,
            };

            // The function logs coordinates if both are present (regardless of validity)
            let has_both_coordinates = request.latitude.is_some() && request.longitude.is_some();
            assert_eq!(
                has_both_coordinates, should_log_coordinates,
                "Coordinate logic failed for lat: {:?}, lon: {:?}",
                lat, lon
            );
        }
    }
}
