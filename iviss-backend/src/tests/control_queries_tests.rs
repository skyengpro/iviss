//! Integration tests for control_queries module.
//!
//! Tests the repository functions in `crate::queries::control_queries`:
//! - create_control_record
//! - get_control_records

use crate::dto::common::{IdentificationMode, Status};
use crate::dto::create_control::CreateControlRequest;
use crate::dto::list_control::{ActionType, ControlResults};
use crate::queries::control_queries;
use sqlx::postgres::PgPoolOptions;
use testcontainers::{
    runners::AsyncRunner,
};
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

/// Helper: sets up a real Postgres + Moka cache for integration tests.
async fn setup_test_infrastructure() -> (
    sqlx::PgPool,
    testcontainers::ContainerAsync<Postgres>,
) {
    let pg = Postgres::default()
        .with_host_auth()
        .start()
        .await
        .expect("Failed to start Postgres");
    let pg_port = pg
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get Postgres port");
    let db_url = format!("postgres://postgres@127.0.0.1:{}/postgres", pg_port);

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");


    (db, pg)
}

/// Helper: seed an organization.
async fn seed_organization(db: &sqlx::PgPool, name: &str) -> Uuid {
    let org_id = Uuid::new_v4();
    sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
        .bind(org_id)
        .bind(name)
        .bind("police")
        .execute(db)
        .await
        .expect("Failed to seed organization");
    org_id
}

/// Helper: seed an agent user.
async fn seed_agent(db: &sqlx::PgPool, org_id: Uuid, username: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, 'agent'::user_role, $5, $6, $7, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind(username)
    .bind(format!("{}@example.com", username))
    .bind(format!("BADGE-{}", username.to_uppercase()))
    .bind(format!("Agent {}", username))
    .bind(format!("+237600000{}", username.chars().last().unwrap_or('0')))
    .execute(db)
    .await
    .expect("Failed to seed agent");
    user_id
}

/// Helper: create a valid control request.
fn create_control_request(
    agent_id: Uuid,
    org_id: Uuid,
    plate_number: &str,
    identification_mode: IdentificationMode,
    results: ControlResults,
) -> CreateControlRequest {
    CreateControlRequest {
        plate_number: plate_number.to_string(),
        agent_id,
        organization_id: org_id,
        latitude: Some(4.0511),
        longitude: Some(9.7679),
        address: Some("Test Address".to_string()),
        identification_mode,
        ocr_confidence: Some(85.0),
        results,
        notes: Some("Test notes".to_string()),
    }
}

/// Helper: seed a control record directly via SQL (for testing get_control_records).
async fn seed_control_record_sql(
    db: &sqlx::PgPool,
    agent_id: Uuid,
    org_id: Uuid,
    plate_number: &str,
    status: &str,
) -> Uuid {
    let control_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address, identification_mode)
        VALUES ($1, $2, $3, $4, NOW(), $5, 'Test Address', 'manual')
        "#,
    )
    .bind(control_id)
    .bind(agent_id)
    .bind(org_id)
    .bind(plate_number)
    .bind(status)
    .execute(db)
    .await
    .expect("Failed to seed control record");
    control_id
}

/// Helper: seed a control action.
async fn seed_control_action(
    db: &sqlx::PgPool,
    control_id: Uuid,
    action_type: &str,
    description: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO control_actions (id, control_id, action_type, description, timestamp)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(control_id)
    .bind(action_type)
    .bind(description)
    .execute(db)
    .await
    .expect("Failed to seed control action");
}

// ─────────────────────────────────────────────────────────────────────────────
// create_control_record Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_control_record_critical_status_wanted() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Critical Org").await;
    let agent_id = seed_agent(&db, org_id, "crit_agent").await;

    let req = create_control_request(
        agent_id,
        org_id,
        "CRIT-001",
        IdentificationMode::Photo,
        ControlResults {
            registration: Status::Valid,
            insurance: Status::Valid,
            technical_inspection: Status::Valid,
            wanted_status: Status::Critical, // Critical wanted status
            customs_status: Status::Valid,
        },
    );

    let result = control_queries::create_control_record(&db, req).await;

    assert!(result.is_ok(), "create_control_record should succeed");
    let control_id = result.unwrap();

    // Verify the control record was created with critical status
    let status: (String,) =
        sqlx::query_as(r#"SELECT overall_status FROM control_records WHERE id = $1"#)
            .bind(control_id)
            .fetch_one(&db)
            .await
            .expect("Failed to fetch control record");
    assert_eq!(status.0, "critical");
}

#[tokio::test]
async fn test_create_control_record_critical_status_insurance() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Insurance Crit Org").await;
    let agent_id = seed_agent(&db, org_id, "ins_crit_agent").await;

    let req = create_control_request(
        agent_id,
        org_id,
        "INS-CRIT",
        IdentificationMode::Live,
        ControlResults {
            registration: Status::Valid,
            insurance: Status::Critical, // Critical insurance status
            technical_inspection: Status::Valid,
            wanted_status: Status::Valid,
            customs_status: Status::Valid,
        },
    );

    let result = control_queries::create_control_record(&db, req).await;

    assert!(result.is_ok(), "create_control_record should succeed");
    let control_id = result.unwrap();

    let status: (String,) =
        sqlx::query_as(r#"SELECT overall_status FROM control_records WHERE id = $1"#)
            .bind(control_id)
            .fetch_one(&db)
            .await
            .expect("Failed to fetch control record");
    assert_eq!(status.0, "critical");
}

#[tokio::test]
async fn test_create_control_record_warning_status_technical() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Warning Org").await;
    let agent_id = seed_agent(&db, org_id, "warn_agent").await;

    let req = create_control_request(
        agent_id,
        org_id,
        "WARN-001",
        IdentificationMode::Manual,
        ControlResults {
            registration: Status::Valid,
            insurance: Status::Valid,
            technical_inspection: Status::Warning, // Warning status
            wanted_status: Status::Valid,
            customs_status: Status::Valid,
        },
    );

    let result = control_queries::create_control_record(&db, req).await;

    assert!(result.is_ok(), "create_control_record should succeed");
    let control_id = result.unwrap();

    let status: (String,) =
        sqlx::query_as(r#"SELECT overall_status FROM control_records WHERE id = $1"#)
            .bind(control_id)
            .fetch_one(&db)
            .await
            .expect("Failed to fetch control record");
    assert_eq!(status.0, "warning");
}

#[tokio::test]
async fn test_create_control_record_warning_status_customs() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Customs Warn Org").await;
    let agent_id = seed_agent(&db, org_id, "customs_warn_agent").await;

    let req = create_control_request(
        agent_id,
        org_id,
        "CUST-WARN",
        IdentificationMode::Photo,
        ControlResults {
            registration: Status::Valid,
            insurance: Status::Valid,
            technical_inspection: Status::Valid,
            wanted_status: Status::Valid,
            customs_status: Status::Warning, // Warning status
        },
    );

    let result = control_queries::create_control_record(&db, req).await;

    assert!(result.is_ok(), "create_control_record should succeed");
    let control_id = result.unwrap();

    let status: (String,) =
        sqlx::query_as(r#"SELECT overall_status FROM control_records WHERE id = $1"#)
            .bind(control_id)
            .fetch_one(&db)
            .await
            .expect("Failed to fetch control record");
    assert_eq!(status.0, "warning");
}

#[tokio::test]
async fn test_create_control_record_valid_status() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Valid Org").await;
    let agent_id = seed_agent(&db, org_id, "valid_agent").await;

    let req = create_control_request(
        agent_id,
        org_id,
        "VALID-001",
        IdentificationMode::Live,
        ControlResults {
            registration: Status::Valid,
            insurance: Status::Valid,
            technical_inspection: Status::Valid,
            wanted_status: Status::Valid,
            customs_status: Status::Valid, // All valid
        },
    );

    let result = control_queries::create_control_record(&db, req).await;

    assert!(result.is_ok(), "create_control_record should succeed");
    let control_id = result.unwrap();

    let status: (String,) =
        sqlx::query_as(r#"SELECT overall_status FROM control_records WHERE id = $1"#)
            .bind(control_id)
            .fetch_one(&db)
            .await
            .expect("Failed to fetch control record");
    assert_eq!(status.0, "valid");
}

#[tokio::test]
async fn test_create_control_record_creates_initial_action() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Action Test Org").await;
    let agent_id = seed_agent(&db, org_id, "action_agent").await;

    let req = create_control_request(
        agent_id,
        org_id,
        "ACTION-001",
        IdentificationMode::Photo,
        ControlResults {
            registration: Status::Valid,
            insurance: Status::Valid,
            technical_inspection: Status::Valid,
            wanted_status: Status::Valid,
            customs_status: Status::Valid,
        },
    );

    let result = control_queries::create_control_record(&db, req).await;

    assert!(result.is_ok(), "create_control_record should succeed");
    let control_id = result.unwrap();

    // Verify initial action was created
    let actions: Vec<(String,)> =
        sqlx::query_as(r#"SELECT action_type FROM control_actions WHERE control_id = $1"#)
            .bind(control_id)
            .fetch_all(&db)
            .await
            .expect("Failed to fetch actions");

    assert_eq!(actions.len(), 1, "Should have exactly 1 action");
    assert_eq!(actions[0].0, "flag", "Initial action should be 'flag'");
}

// ─────────────────────────────────────────────────────────────────────────────
// get_control_records Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_control_records_no_filters() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "List Test Org").await;
    let agent_id = seed_agent(&db, org_id, "list_agent").await;

    // Seed multiple control records
    seed_control_record_sql(&db, agent_id, org_id, "LIST-001", "valid").await;
    seed_control_record_sql(&db, agent_id, org_id, "LIST-002", "warning").await;
    seed_control_record_sql(&db, agent_id, org_id, "LIST-003", "critical").await;

    let result = control_queries::get_control_records(&db, None, None, None, None, None).await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 3, "Should return all 3 control records");
}

#[tokio::test]
async fn test_get_control_records_filter_by_agent_id() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Agent Filter Org").await;
    let agent1_id = seed_agent(&db, org_id, "agent_filter_1").await;
    let agent2_id = seed_agent(&db, org_id, "agent_filter_2").await;

    // Agent 1 has 2 records
    seed_control_record_sql(&db, agent1_id, org_id, "AGENT1-001", "valid").await;
    seed_control_record_sql(&db, agent1_id, org_id, "AGENT1-002", "warning").await;
    // Agent 2 has 1 record
    seed_control_record_sql(&db, agent2_id, org_id, "AGENT2-001", "valid").await;

    let result =
        control_queries::get_control_records(&db, None, None, Some(agent1_id), None, None).await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 2, "Should return only agent 1's controls");
    for control in &controls {
        assert_eq!(control.agent_id, agent1_id);
    }
}

#[tokio::test]
async fn test_get_control_records_filter_by_status() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Status Filter Org").await;
    let agent_id = seed_agent(&db, org_id, "status_filter_agent").await;

    seed_control_record_sql(&db, agent_id, org_id, "STAT-001", "valid").await;
    seed_control_record_sql(&db, agent_id, org_id, "STAT-002", "warning").await;
    seed_control_record_sql(&db, agent_id, org_id, "STAT-003", "critical").await;
    seed_control_record_sql(&db, agent_id, org_id, "STAT-004", "valid").await;

    let result =
        control_queries::get_control_records(&db, None, None, None, Some(Status::Critical), None)
            .await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 1, "Should return only 1 critical record");
    assert_eq!(controls[0].status, Status::Critical);
}

#[tokio::test]
async fn test_get_control_records_filter_by_plate() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Plate Filter Org").await;
    let agent_id = seed_agent(&db, org_id, "plate_filter_agent").await;

    seed_control_record_sql(&db, agent_id, org_id, "XY-1234-AB", "valid").await;
    seed_control_record_sql(&db, agent_id, org_id, "XY-5678-CD", "valid").await;
    seed_control_record_sql(&db, agent_id, org_id, "ZZ-9999-YY", "valid").await;

    let result =
        control_queries::get_control_records(&db, None, None, None, None, Some("XY".to_string()))
            .await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 2, "Should return 2 records matching 'XY'");
    for control in &controls {
        assert!(control.plate_number.contains("XY"));
    }
}

#[tokio::test]
async fn test_get_control_records_multiple_filters() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Multi Filter Org").await;
    let agent_id = seed_agent(&db, org_id, "multi_filter_agent").await;

    seed_control_record_sql(&db, agent_id, org_id, "MULTI-001", "valid").await;
    seed_control_record_sql(&db, agent_id, org_id, "MULTI-002", "warning").await;
    seed_control_record_sql(&db, agent_id, org_id, "MULTI-003", "critical").await;

    // Filter by status and plate
    let result = control_queries::get_control_records(
        &db,
        None,
        None,
        None,
        Some(Status::Warning),
        Some("MULTI".to_string()),
    )
    .await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 1);
    assert_eq!(controls[0].status, Status::Warning);
    assert!(controls[0].plate_number.contains("MULTI"));
}

#[tokio::test]
async fn test_get_control_records_empty_result() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Empty Result Org").await;
    let agent_id = seed_agent(&db, org_id, "empty_result_agent").await;

    seed_control_record_sql(&db, agent_id, org_id, "EXISTING", "valid").await;

    // Search for non-existent plate
    let result = control_queries::get_control_records(
        &db,
        None,
        None,
        None,
        None,
        Some("NONEXISTENT".to_string()),
    )
    .await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert!(controls.is_empty(), "Should return empty result");
}

#[tokio::test]
async fn test_get_control_records_excludes_deleted() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Deleted Filter Org").await;
    let agent_id = seed_agent(&db, org_id, "deleted_filter_agent").await;

    // Create active record
    seed_control_record_sql(&db, agent_id, org_id, "ACTIVE-REC", "valid").await;

    // Create and delete a record
    let deleted_id = seed_control_record_sql(&db, agent_id, org_id, "DELETED-REC", "valid").await;
    sqlx::query(r#"UPDATE control_records SET deleted_at = NOW() WHERE id = $1"#)
        .bind(deleted_id)
        .execute(&db)
        .await
        .expect("Failed to soft-delete record");

    let result = control_queries::get_control_records(&db, None, None, None, None, None).await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 1, "Should exclude deleted records");
    assert_eq!(controls[0].plate_number, "ACTIVE-REC");
}

#[tokio::test]
async fn test_get_control_records_includes_actions() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Actions Test Org").await;
    let agent_id = seed_agent(&db, org_id, "actions_test_agent").await;

    // Create a control record
    let control_id = seed_control_record_sql(&db, agent_id, org_id, "ACTION-REC", "valid").await;

    // Add multiple actions - valid DB values: citation, impound, flag, warning
    seed_control_action(&db, control_id, "flag", "Initial check performed").await;
    seed_control_action(&db, control_id, "citation", "Citation issued").await;

    let result = control_queries::get_control_records(
        &db,
        None,
        None,
        None,
        None,
        Some("ACTION-REC".to_string()),
    )
    .await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 1);
    assert_eq!(controls[0].actions.len(), 2, "Should include 2 actions");
    // Check action types using matches! - valid DB values: citation, impound, flag, warning
    assert!(matches!(
        controls[0].actions[0].action_type,
        ActionType::Flag
    ));
    assert!(matches!(
        controls[0].actions[1].action_type,
        ActionType::Citation
    ));
}

#[tokio::test]
async fn test_get_control_records_with_all_identification_modes() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "ID Mode Org").await;
    let agent_id = seed_agent(&db, org_id, "id_mode_agent").await;

    // Create control records with different identification modes directly
    let control1_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, identification_mode)
           VALUES ($1, $2, $3, $4, NOW(), 'valid', 'manual')"#,
    )
    .bind(control1_id)
    .bind(agent_id)
    .bind(org_id)
    .bind("MANUAL-001")
    .execute(&db)
    .await
    .expect("Failed to seed");

    let control2_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, identification_mode)
           VALUES ($1, $2, $3, $4, NOW(), 'valid', 'photo')"#,
    )
    .bind(control2_id)
    .bind(agent_id)
    .bind(org_id)
    .bind("PHOTO-001")
    .execute(&db)
    .await
    .expect("Failed to seed");

    let control3_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, identification_mode)
           VALUES ($1, $2, $3, $4, NOW(), 'valid', 'live')"#,
    )
    .bind(control3_id)
    .bind(agent_id)
    .bind(org_id)
    .bind("LIVE-001")
    .execute(&db)
    .await
    .expect("Failed to seed");

    let result =
        control_queries::get_control_records(&db, None, None, Some(agent_id), None, None).await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 3);

    // Check identification modes using matches!
    let has_manual = controls
        .iter()
        .any(|c| matches!(c.identification_mode, IdentificationMode::Manual));
    let has_photo = controls
        .iter()
        .any(|c| matches!(c.identification_mode, IdentificationMode::Photo));
    let has_live = controls
        .iter()
        .any(|c| matches!(c.identification_mode, IdentificationMode::Live));
    assert!(has_manual, "Should have a Manual identification mode");
    assert!(has_photo, "Should have a Photo identification mode");
    assert!(has_live, "Should have a Live identification mode");
}

#[tokio::test]
async fn test_get_control_records_results_parsing() {
    let (db, _pg) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Results Test Org").await;
    let agent_id = seed_agent(&db, org_id, "results_test_agent").await;

    // Create a control record with results JSON
    let control_id = Uuid::new_v4();
    let results_json = serde_json::json!({
        "registration": "valid",
        "insurance": "warning",
        "technical_inspection": "valid",
        "wanted_status": "valid",
        "customs_status": "critical"
    });

    sqlx::query(
        r#"INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, results_json, identification_mode)
           VALUES ($1, $2, $3, $4, NOW(), 'warning', $5, 'manual')"#,
    )
    .bind(control_id)
    .bind(agent_id)
    .bind(org_id)
    .bind("RESULTS-001")
    .bind(&results_json)
    .execute(&db)
    .await
    .expect("Failed to seed");

    let result = control_queries::get_control_records(
        &db,
        None,
        None,
        None,
        None,
        Some("RESULTS".to_string()),
    )
    .await;

    assert!(result.is_ok(), "get_control_records should succeed");
    let controls = result.unwrap();

    assert_eq!(controls.len(), 1);
    assert_eq!(controls[0].results.insurance, Status::Warning);
    assert_eq!(controls[0].results.customs_status, Status::Critical);
    assert_eq!(controls[0].status, Status::Warning); // Overall status is warning
}
