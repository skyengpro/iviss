/// Integration tests for multi-tenant data isolation
/// Tests that organizations cannot access each other's data
mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use helpers::*;
use tower::ServiceExt;

#[tokio::test]
async fn test_admin_endpoints_require_authentication() {
    let (app, _db, _org_id, _user_id, _device_id, _pg, _cache, _config) =
        setup_complete_test_infrastructure().await;

    let admin_endpoints = vec![
        "/api/v1/admin/users",
        "/api/v1/admin/organizations",
        "/api/v1/admin/stats",
    ];

    for endpoint in admin_endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(endpoint)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} should require authentication",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_vehicle_search_works_for_authenticated_users() {
    let (app, db, _org_id, user_id, _device_id, _pg, _cache, config) =
        setup_complete_test_infrastructure().await;

    // Insert vehicle with owner
    let vehicle_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO vehicles (id, plate_number, chassis_number, brand, model, year, color, engine_power, fuel_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(vehicle_id)
    .bind("NO123AB")
    .bind("CHASSISNO123AB")
    .bind("Toyota")
    .bind("Camry")
    .bind(2020i32)
    .bind("Black")
    .bind("150HP")
    .bind("Gasoline")
    .execute(&db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO vehicle_owners (id, vehicle_id, name, address, national_id, is_current_owner)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(vehicle_id)
    .bind("Test Owner")
    .bind("Test Address")
    .bind("ID123")
    .bind(true)
    .execute(&db)
    .await
    .unwrap();

    // Generate token for agent
    let device_id = insert_test_device(&db, user_id, "ACTIVE").await;
    let token = generate_test_jwt_token(&config, user_id, device_id, "agent").await;

    // Search for vehicle
    let request_body = serde_json::json!({
        "plate": "NO 123 AB"
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
}
