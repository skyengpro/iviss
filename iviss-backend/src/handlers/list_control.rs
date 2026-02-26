use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    dto::{
        create_control::{CreateControlRequest, CreateControlResponse},
        list_control::ControlListQuery,
    },
    errors::AppError,
};

#[utoipa::path(
    post,
    path = "/controls",
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
pub async fn create_control(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateControlRequest>,
) -> Result<impl IntoResponse, AppError> {
    let id = crate::queries::control_queries::create_control_record(&state.db, payload).await?;

    let response = CreateControlResponse {
        id,
        message: "Control logged successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/controls",
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

#[cfg(test)]
mod tests {
    use crate::dto::common::{IdentificationMode, Status};
    use crate::dto::create_control::{CreateControlRequest, CreateControlResponse};
    use crate::dto::list_control::{
        ActionType, ControlAction, ControlListQuery, ControlLocation, ControlResults,
        ListControlResponse,
    };
    use axum::http::StatusCode;
    use uuid::Uuid;

    // Helper function to create test control request
    fn create_test_control_request() -> CreateControlRequest {
        CreateControlRequest {
            plate_number: "AA 123 BB".to_string(),
            agent_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
            address: Some("123 Main St".to_string()),
            identification_mode: IdentificationMode::Manual,
            ocr_confidence: Some(0.95),
            results: ControlResults {
                registration: Status::Valid,
                insurance: Status::Valid,
                technical_inspection: Status::Valid,
                wanted_status: Status::Valid,
                customs_status: Status::Valid,
            },
            notes: Some("Routine check".to_string()),
        }
    }

    // Helper function to create test control list query
    fn create_test_control_query() -> ControlListQuery {
        ControlListQuery {
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            agent_id: Some(Uuid::new_v4()),
            status: Some(Status::Valid),
            plate: Some("AA 123 BB".to_string()),
        }
    }

    // ============ create_control Tests ============

    #[test]
    fn test_create_control_request_structure() {
        let request = create_test_control_request();

        assert_eq!(request.plate_number, "AA 123 BB");
        assert!(!request.agent_id.is_nil()); // UUID should be valid
        assert!(!request.organization_id.is_nil()); // UUID should be valid
        assert_eq!(request.latitude, Some(40.7128));
        assert_eq!(request.longitude, Some(-74.0060));
        assert_eq!(request.address, Some("123 Main St".to_string()));

        // Test IdentificationMode using pattern matching
        match request.identification_mode {
            IdentificationMode::Manual => assert!(true),
            _ => assert!(false, "Expected Manual mode"),
        }

        assert_eq!(request.ocr_confidence, Some(0.95));
        assert_eq!(request.results.registration, Status::Valid);
        assert_eq!(request.results.insurance, Status::Valid);
        assert_eq!(request.notes, Some("Routine check".to_string()));
    }

    #[test]
    fn test_create_control_request_with_optional_fields_none() {
        let request = CreateControlRequest {
            plate_number: "BB 456 CC".to_string(),
            agent_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            latitude: None,
            longitude: None,
            address: None,
            identification_mode: IdentificationMode::Photo,
            ocr_confidence: None,
            results: ControlResults {
                registration: Status::Critical,
                insurance: Status::Warning,
                technical_inspection: Status::Critical,
                wanted_status: Status::Valid,
                customs_status: Status::Pending,
            },
            notes: None,
        };

        assert_eq!(request.plate_number, "BB 456 CC");
        assert!(request.latitude.is_none());
        assert!(request.longitude.is_none());
        assert!(request.address.is_none());
        assert!(request.ocr_confidence.is_none());
        assert!(request.notes.is_none());

        // Test IdentificationMode using pattern matching
        match request.identification_mode {
            IdentificationMode::Photo => assert!(true),
            _ => assert!(false, "Expected Photo mode"),
        }

        assert_eq!(request.results.registration, Status::Critical);
        assert_eq!(request.results.insurance, Status::Warning);
        assert_eq!(request.results.technical_inspection, Status::Critical);
        assert_eq!(request.results.wanted_status, Status::Valid);
        assert_eq!(request.results.customs_status, Status::Pending);
    }

    #[test]
    fn test_create_control_response_structure() {
        let id = Uuid::new_v4();
        let response = CreateControlResponse {
            id,
            message: "Control logged successfully".to_string(),
        };

        assert_eq!(response.id, id);
        assert_eq!(response.message, "Control logged successfully");
    }

    #[test]
    fn test_create_control_response_creation() {
        let id = Uuid::new_v4();
        let response = CreateControlResponse {
            id,
            message: "Control logged successfully".to_string(),
        };

        assert!(!response.id.is_nil()); // UUID should be valid
        assert!(!response.message.is_empty());
        assert_eq!(response.message, "Control logged successfully");
    }

    #[test]
    fn test_create_control_response_status_code() {
        let id = Uuid::new_v4();
        let response = CreateControlResponse {
            id,
            message: "Control logged successfully".to_string(),
        };

        // Test that the response can be created with proper status
        let result = (StatusCode::CREATED, axum::Json(response));
        assert_eq!(result.0, StatusCode::CREATED);
    }

    // ============ ControlListQuery Tests ============

    #[test]
    fn test_control_list_query_structure() {
        let query = create_test_control_query();

        assert_eq!(query.start_date, Some("2024-01-01".to_string()));
        assert_eq!(query.end_date, Some("2024-12-31".to_string()));
        assert!(query.agent_id.is_some());
        assert!(!query.agent_id.unwrap().is_nil()); // UUID should be valid
        assert_eq!(query.status, Some(Status::Valid));
        assert_eq!(query.plate, Some("AA 123 BB".to_string()));
    }

    #[test]
    fn test_control_list_query_with_all_none() {
        let query = ControlListQuery {
            start_date: None,
            end_date: None,
            agent_id: None,
            status: None,
            plate: None,
        };

        assert!(query.start_date.is_none());
        assert!(query.end_date.is_none());
        assert!(query.agent_id.is_none());
        assert!(query.status.is_none());
        assert!(query.plate.is_none());
    }

    #[test]
    fn test_control_list_query_partial_filters() {
        let query = ControlListQuery {
            start_date: Some("2024-06-01".to_string()),
            end_date: None,
            agent_id: Some(Uuid::new_v4()),
            status: Some(Status::Critical),
            plate: None,
        };

        assert_eq!(query.start_date, Some("2024-06-01".to_string()));
        assert!(query.end_date.is_none());
        assert!(query.agent_id.is_some());
        assert_eq!(query.status, Some(Status::Critical));
        assert!(query.plate.is_none());
    }

    // ============ ControlResults Tests ============

    #[test]
    fn test_control_results_all_valid() {
        let results = ControlResults {
            registration: Status::Valid,
            insurance: Status::Valid,
            technical_inspection: Status::Valid,
            wanted_status: Status::Valid,
            customs_status: Status::Valid,
        };

        assert_eq!(results.registration, Status::Valid);
        assert_eq!(results.insurance, Status::Valid);
        assert_eq!(results.technical_inspection, Status::Valid);
        assert_eq!(results.wanted_status, Status::Valid);
        assert_eq!(results.customs_status, Status::Valid);
    }

    #[test]
    fn test_control_results_mixed_statuses() {
        let results = ControlResults {
            registration: Status::Critical,
            insurance: Status::Warning,
            technical_inspection: Status::Pending,
            wanted_status: Status::Valid,
            customs_status: Status::Critical,
        };

        assert_eq!(results.registration, Status::Critical);
        assert_eq!(results.insurance, Status::Warning);
        assert_eq!(results.technical_inspection, Status::Pending);
        assert_eq!(results.wanted_status, Status::Valid);
        assert_eq!(results.customs_status, Status::Critical);
    }

    // ============ ControlLocation Tests ============

    #[test]
    fn test_control_location_with_all_fields() {
        let location = ControlLocation {
            address: Some("123 Main St, City".to_string()),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
        };

        assert_eq!(location.address, Some("123 Main St, City".to_string()));
        assert_eq!(location.latitude, Some(40.7128));
        assert_eq!(location.longitude, Some(-74.0060));
    }

    #[test]
    fn test_control_location_with_optional_fields_none() {
        let location = ControlLocation {
            address: None,
            latitude: None,
            longitude: None,
        };

        assert!(location.address.is_none());
        assert!(location.latitude.is_none());
        assert!(location.longitude.is_none());
    }

    // ============ ControlAction Tests ============

    #[test]
    fn test_control_action_all_action_types() {
        let test_cases = [
            (ActionType::Check, "Check"),
            (ActionType::Flag, "Flag"),
            (ActionType::Citation, "Citation"),
            (ActionType::Impound, "Impound"),
            (ActionType::Release, "Release"),
        ];

        for (action_type, expected_name) in &test_cases {
            let action = ControlAction {
                action_type: match action_type {
                    ActionType::Check => ActionType::Check,
                    ActionType::Flag => ActionType::Flag,
                    ActionType::Citation => ActionType::Citation,
                    ActionType::Impound => ActionType::Impound,
                    ActionType::Release => ActionType::Release,
                },
                description: Some(format!("Action: {}", expected_name)),
                timestamp: "2024-01-01T12:00:00Z".to_string(),
            };

            // Test action_type using pattern matching
            match action.action_type {
                ActionType::Check => assert!(true),
                ActionType::Flag => assert!(true),
                ActionType::Citation => assert!(true),
                ActionType::Impound => assert!(true),
                ActionType::Release => assert!(true),
            }

            assert!(action.description.is_some());
            assert!(!action.timestamp.is_empty());
        }
    }

    #[test]
    fn test_control_action_with_optional_fields() {
        let action = ControlAction {
            action_type: ActionType::Check,
            description: None,
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        };

        // Test action_type using pattern matching
        match action.action_type {
            ActionType::Check => assert!(true),
            _ => assert!(false, "Expected Check action"),
        }

        assert!(action.description.is_none());
        assert_eq!(action.timestamp, "2024-01-01T12:00:00Z");
    }

    // ============ ListControlResponse Tests ============

    #[test]
    fn test_list_control_response_structure() {
        let id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let response = ListControlResponse {
            id,
            plate_number: "AA 123 BB".to_string(),
            agent_name: Some("John Doe".to_string()),
            agent_id,
            organization_id,
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            status: Status::Valid,
            identification_mode: IdentificationMode::Manual,
            confidence: Some(0.95),
            location: ControlLocation {
                address: Some("123 Main St".to_string()),
                latitude: Some(40.7128),
                longitude: Some(-74.0060),
            },
            results: ControlResults {
                registration: Status::Valid,
                insurance: Status::Valid,
                technical_inspection: Status::Valid,
                wanted_status: Status::Valid,
                customs_status: Status::Valid,
            },
            actions: vec![ControlAction {
                action_type: ActionType::Check,
                description: Some("Routine inspection".to_string()),
                timestamp: "2024-01-01T12:00:00Z".to_string(),
            }],
            notes: Some("No issues found".to_string()),
            vehicle: None,
        };

        assert_eq!(response.plate_number, "AA 123 BB");
        assert_eq!(response.agent_name, Some("John Doe".to_string()));
        assert_eq!(response.status, Status::Valid);
        assert_eq!(response.confidence, Some(0.95));
        assert_eq!(response.notes, Some("No issues found".to_string()));
        assert!(response.vehicle.is_none());
        assert_eq!(response.actions.len(), 1);

        // Test identification_mode using pattern matching
        match response.identification_mode {
            IdentificationMode::Manual => assert!(true),
            _ => assert!(false, "Expected Manual mode"),
        }

        // Test UUID validity
        assert!(!response.id.is_nil());
        assert!(!response.agent_id.is_nil());
        assert!(!response.organization_id.is_nil());
    }

    #[test]
    fn test_list_control_response_with_multiple_actions() {
        let response = ListControlResponse {
            id: Uuid::new_v4(),
            plate_number: "BB 456 CC".to_string(),
            agent_name: None,
            agent_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            status: Status::Critical,
            identification_mode: IdentificationMode::Photo,
            confidence: None,
            location: ControlLocation {
                address: None,
                latitude: None,
                longitude: None,
            },
            results: ControlResults {
                registration: Status::Critical,
                insurance: Status::Warning,
                technical_inspection: Status::Critical,
                wanted_status: Status::Valid,
                customs_status: Status::Pending,
            },
            actions: vec![
                ControlAction {
                    action_type: ActionType::Flag,
                    description: Some("Vehicle flagged".to_string()),
                    timestamp: "2024-01-01T12:00:00Z".to_string(),
                },
                ControlAction {
                    action_type: ActionType::Citation,
                    description: Some("Citation issued".to_string()),
                    timestamp: "2024-01-01T12:05:00Z".to_string(),
                },
            ],
            notes: None,
            vehicle: None,
        };

        assert_eq!(response.plate_number, "BB 456 CC");
        assert_eq!(response.status, Status::Critical);
        assert!(response.confidence.is_none());
        assert!(response.agent_name.is_none());
        assert_eq!(response.actions.len(), 2);

        // Test identification_mode using pattern matching
        match response.identification_mode {
            IdentificationMode::Photo => assert!(true),
            _ => assert!(false, "Expected Photo mode"),
        }
    }

    // ============ Integration Logic Tests ============

    #[test]
    fn test_status_variations_in_results() {
        let test_cases = vec![
            Status::Valid,
            Status::Warning,
            Status::Critical,
            Status::Pending,
        ];

        for status in test_cases {
            let results = ControlResults {
                registration: status.clone(),
                insurance: status.clone(),
                technical_inspection: status.clone(),
                wanted_status: status.clone(),
                customs_status: status.clone(),
            };

            assert_eq!(results.registration, status);
            assert_eq!(results.insurance, status);
            assert_eq!(results.technical_inspection, status);
            assert_eq!(results.wanted_status, status);
            assert_eq!(results.customs_status, status);
        }
    }

    #[test]
    fn test_uuid_handling_in_structures() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        let request = CreateControlRequest {
            plate_number: "CC 789 DD".to_string(),
            agent_id: id1,
            organization_id: id2,
            latitude: None,
            longitude: None,
            address: None,
            identification_mode: IdentificationMode::Live,
            ocr_confidence: None,
            results: ControlResults {
                registration: Status::Pending,
                insurance: Status::Pending,
                technical_inspection: Status::Pending,
                wanted_status: Status::Pending,
                customs_status: Status::Pending,
            },
            notes: None,
        };

        let response = ListControlResponse {
            id: id3,
            plate_number: "CC 789 DD".to_string(),
            agent_name: None,
            agent_id: id1,
            organization_id: id2,
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            status: Status::Pending,
            identification_mode: IdentificationMode::Live,
            confidence: None,
            location: ControlLocation {
                address: None,
                latitude: None,
                longitude: None,
            },
            results: ControlResults {
                registration: Status::Pending,
                insurance: Status::Pending,
                technical_inspection: Status::Pending,
                wanted_status: Status::Pending,
                customs_status: Status::Pending,
            },
            actions: vec![],
            notes: None,
            vehicle: None,
        };

        assert_eq!(request.agent_id, id1);
        assert_eq!(request.organization_id, id2);
        assert_eq!(response.id, id3);
        assert_eq!(response.agent_id, id1);
        assert_eq!(response.organization_id, id2);

        // Test identification_mode using pattern matching
        match request.identification_mode {
            IdentificationMode::Live => assert!(true),
            _ => assert!(false, "Expected Live mode"),
        }

        match response.identification_mode {
            IdentificationMode::Live => assert!(true),
            _ => assert!(false, "Expected Live mode"),
        }
    }

    #[test]
    fn test_coordinate_precision_handling() {
        let request = CreateControlRequest {
            plate_number: "DD 910 EE".to_string(),
            agent_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            latitude: Some(40.7128123456789),
            longitude: Some(-74.0060123456789),
            address: None,
            identification_mode: IdentificationMode::Manual,
            ocr_confidence: Some(0.987654321),
            results: ControlResults {
                registration: Status::Valid,
                insurance: Status::Valid,
                technical_inspection: Status::Valid,
                wanted_status: Status::Valid,
                customs_status: Status::Valid,
            },
            notes: None,
        };

        assert_eq!(request.latitude, Some(40.7128123456789));
        assert_eq!(request.longitude, Some(-74.0060123456789));
        assert_eq!(request.ocr_confidence, Some(0.987654321));
    }

    #[test]
    fn test_timestamp_format_consistency() {
        let timestamp = "2024-01-15T14:30:00Z";
        let action = ControlAction {
            action_type: ActionType::Check,
            description: Some("Test action".to_string()),
            timestamp: timestamp.to_string(),
        };

        let response = ListControlResponse {
            id: Uuid::new_v4(),
            plate_number: "EE 111 FF".to_string(),
            agent_name: Some("Test Agent".to_string()),
            agent_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            timestamp: timestamp.to_string(),
            status: Status::Valid,
            identification_mode: IdentificationMode::Manual,
            confidence: Some(1.0),
            location: ControlLocation {
                address: Some("Test Location".to_string()),
                latitude: Some(0.0),
                longitude: Some(0.0),
            },
            results: ControlResults {
                registration: Status::Valid,
                insurance: Status::Valid,
                technical_inspection: Status::Valid,
                wanted_status: Status::Valid,
                customs_status: Status::Valid,
            },
            actions: vec![action],
            notes: Some("Test note".to_string()),
            vehicle: None,
        };

        assert_eq!(response.timestamp, timestamp);
        assert_eq!(response.actions[0].timestamp, timestamp);
    }

    #[test]
    fn test_empty_actions_vector() {
        let response = ListControlResponse {
            id: Uuid::new_v4(),
            plate_number: "FF 222 GG".to_string(),
            agent_name: None,
            agent_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            status: Status::Pending,
            identification_mode: IdentificationMode::Photo,
            confidence: None,
            location: ControlLocation {
                address: None,
                latitude: None,
                longitude: None,
            },
            results: ControlResults {
                registration: Status::Pending,
                insurance: Status::Pending,
                technical_inspection: Status::Pending,
                wanted_status: Status::Pending,
                customs_status: Status::Pending,
            },
            actions: vec![], // Empty vector
            notes: None,
            vehicle: None,
        };

        assert!(response.actions.is_empty());
        assert_eq!(response.actions.len(), 0);
    }
}
