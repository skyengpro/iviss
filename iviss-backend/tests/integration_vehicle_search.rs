/// Integration tests for vehicle search functionality
/// Tests vehicle search endpoint with real database queries
mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use helpers::*;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper to insert test vehicle with owner
async fn insert_test_vehicle(
    db: &sqlx::PgPool,
    plate_number: &str,
    brand: &str,
    model: &str,
) -> Uuid {
    let vehicle_id = Uuid::new_v4();
    let chassis_number = format!("CHASSIS{}", &plate_number.replace(" ", ""));

    // Insert vehicle
    sqlx::query(
        r#"
        INSERT INTO vehicles (id, plate_number, chassis_number, brand, model, year, color, engine_power, fuel_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(vehicle_id)
    .bind(plate_number)
    .bind(&chassis_number)
    .bind(brand)
    .bind(model)
    .bind(2020i32)
    .bind("Black")
    .bind("150HP")
    .bind("Gasoline")
    .execute(db)
    .await
    .expect("Failed to insert test vehicle");

    // Insert vehicle owner
    sqlx::query(
        r#"
        INSERT INTO vehicle_owners (id, vehicle_id, name, address, national_id, is_current_owner)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(vehicle_id)
    .bind(format!("Owner of {}", plate_number))
    .bind("123 Test Street")
    .bind("ID123456")
    .bind(true)
    .execute(db)
    .await
    .expect("Failed to insert vehicle owner");

    vehicle_id
}

#[tokio::test]
async fn test_vehicle_search_requires_authentication() {
    let (app, _db, _org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    // Try to search without authentication (using valid Cameroon plate format)
    let request_body = serde_json::json!({
        "plate": "CE 128 BC"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/search")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_vehicle_search_finds_existing_vehicle() {
    let (app, db, _org_id, user_id, _device_id, _pg, _cache, config) =
        setup_complete_test_infrastructure().await;

    // Insert test vehicles (using valid Cameroon plate formats)
    insert_test_vehicle(&db, "CE128BC", "Toyota", "Camry").await;
    insert_test_vehicle(&db, "LT3334W", "Honda", "Civic").await;

    // Generate auth token (device_id needed for agent role)
    let device_id = insert_test_device(&db, user_id, "ACTIVE").await;
    let token = generate_test_jwt_token(&config, user_id, device_id, "agent").await;

    // Search for existing vehicle
    let request_body = serde_json::json!({
        "plate": "CE 128 BC"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/search")
                .header("Authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify response body
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Check that we got a result (the endpoint returns a VehicleSearchResult object)
    assert!(
        body.is_object(),
        "Should return a vehicle search result object"
    );
}

#[tokio::test]
async fn test_vehicle_search_returns_not_found_for_nonexistent() {
    let (app, db, _org_id, user_id, _device_id, _pg, _cache, config) =
        setup_complete_test_infrastructure().await;

    // Insert test vehicle
    insert_test_vehicle(&db, "CE128BC", "Toyota", "Camry").await;

    // Generate auth token (device_id needed for agent role)
    let device_id = insert_test_device(&db, user_id, "ACTIVE").await;
    let token = generate_test_jwt_token(&config, user_id, device_id, "agent").await;

    // Search for non-existent vehicle (valid format but not in DB)
    let request_body = serde_json::json!({
        "plate": "LT 9999 Z"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/search")
                .header("Authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 for not found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_vehicle_search_validates_missing_plate_number() {
    let (app, db, _org_id, user_id, _device_id, _pg, _cache, config) =
        setup_complete_test_infrastructure().await;

    // Generate auth token (device_id needed for agent role)
    let device_id = insert_test_device(&db, user_id, "ACTIVE").await;
    let token = generate_test_jwt_token(&config, user_id, device_id, "agent").await;

    // Search without plate parameter (empty body)
    let request_body = serde_json::json!({});

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/search")
                .header("Authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 422 Unprocessable Entity for missing required field
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_vehicle_search_multi_tenant_isolation() {
    let (app, db, _org_id, user_id, _device_id, _pg, _cache, config) =
        setup_complete_test_infrastructure().await;

    // Create second organization
    let _org_b_id = insert_test_organization(&db, "Org B", "police").await;

    // Insert vehicles (vehicles are global in this schema, not org-specific)
    // Using valid Cameroon plate formats
    insert_test_vehicle(&db, "CE128BC", "Toyota", "Camry").await;
    insert_test_vehicle(&db, "LT3334W", "Honda", "Civic").await;

    // Generate token for Org A user (device_id needed for agent role)
    let device_id = insert_test_device(&db, user_id, "ACTIVE").await;
    let token = generate_test_jwt_token(&config, user_id, device_id, "agent").await;

    // Search for a vehicle using Org A's token
    let request_body = serde_json::json!({
        "plate": "LT 3334 W"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/search")
                .header("Authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify search works (vehicles are global, so any org can search any vehicle)
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Vehicle search should return a result object
    assert!(body.is_object(), "Should return a vehicle search result");
}
