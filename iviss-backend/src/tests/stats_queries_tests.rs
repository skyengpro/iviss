//! Integration tests for stats_queries module.
//!
//! Tests the repository functions in `crate::queries::stats_queries`:
//! - get_control_activity_series_query
//! - get_top_agents_query
//! - get_activity_feed_query
//! - get_recent_alerts_query
//! - get_dashboard_stats_query

use crate::dto::stats::DashboardRange;
use crate::queries::stats_queries;
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use uuid::Uuid;

/// Helper: sets up a real Postgres + Redis for integration tests.
async fn setup_test_infrastructure() -> (
    sqlx::PgPool,
    testcontainers::ContainerAsync<Postgres>,
    testcontainers::ContainerAsync<Redis>,
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
    let db_url = format!(
        "postgres://postgres@127.0.0.1:{}/postgres",
        pg_port
    );

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

    // Start Redis (required for migrations even if not used in these tests)
    let redis_container = Redis::default()
        .start()
        .await
        .expect("Failed to start Redis");

    (db, pg, redis_container)
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

/// Helper: seed a vehicle.
async fn seed_vehicle(db: &sqlx::PgPool, plate: &str) -> Uuid {
    let vehicle_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO vehicles (id, plate_number, chassis_number)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(vehicle_id)
    .bind(plate)
    .bind(format!("CHASSIS-{}", Uuid::new_v4()).replace("-", ""))
    .execute(db)
    .await
    .expect("Failed to seed vehicle");
    vehicle_id
}

/// Helper: seed a control record with specific status.
async fn seed_control_record(
    db: &sqlx::PgPool,
    agent_id: Uuid,
    org_id: Uuid,
    plate_number: &str,
    status: &str,
) -> Uuid {
    let control_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, overall_status, address)
        VALUES ($1, $2, $3, $4, NOW(), $5, 'Test Address')
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

/// Helper: seed an agent location.
async fn seed_agent_location(db: &sqlx::PgPool, agent_id: Uuid, latitude: f64, longitude: f64) {
    sqlx::query(
        r#"
        INSERT INTO agent_locations (agent_id, latitude, longitude)
        VALUES ($1, $2, $3)
        ON CONFLICT (agent_id) DO UPDATE SET
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude,
            updated_at = NOW()
        "#,
    )
    .bind(agent_id)
    .bind(latitude)
    .bind(longitude)
    .execute(db)
    .await
    .expect("Failed to seed agent location");
}

/// Helper: seed a pending submission.
async fn seed_pending_submission(
    db: &sqlx::PgPool,
    agent_id: Uuid,
    plate_number: &str,
    status: &str,
) -> Uuid {
    let submission_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO pending_submissions (id, agent_id, plate_number, status)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(submission_id)
    .bind(agent_id)
    .bind(plate_number)
    .bind(status)
    .execute(db)
    .await
    .expect("Failed to seed pending submission");
    submission_id
}

// ─────────────────────────────────────────────────────────────────────────────
// get_control_activity_series_query Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_control_activity_series_h24_with_data() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org H24").await;
    let agent_id = seed_agent(&db, org_id, "agent_h24_1").await;

    // Seed multiple control records
    seed_control_record(&db, agent_id, org_id, "AB-123-CD", "valid").await;
    seed_control_record(&db, agent_id, org_id, "EF-456-GH", "valid").await;
    seed_control_record(&db, agent_id, org_id, "IJ-789-KL", "warning").await;

    let result = stats_queries::get_control_activity_series_query(&db, DashboardRange::H24).await;

    assert!(
        result.is_ok(),
        "get_control_activity_series_query should succeed"
    );
    let series = result.unwrap();

    // H24 should return 24 data points (one per hour)
    assert_eq!(
        series.len(),
        24,
        "H24 range should return 24 hourly buckets"
    );

    // At least one bucket should have data (the current hour)
    let total_count: i64 = series.iter().map(|p| p.count).sum();
    assert_eq!(total_count, 3, "Should have 3 control records in total");
}

#[tokio::test]
async fn test_get_control_activity_series_h24_empty() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    // No control records - should still return 24 buckets with 0 counts
    let result = stats_queries::get_control_activity_series_query(&db, DashboardRange::H24).await;

    assert!(
        result.is_ok(),
        "get_control_activity_series_query should succeed even with no data"
    );
    let series = result.unwrap();

    assert_eq!(
        series.len(),
        24,
        "H24 range should return 24 buckets even with no data"
    );
    for point in &series {
        assert_eq!(point.count, 0, "All counts should be 0 when no data exists");
    }
}

#[tokio::test]
async fn test_get_control_activity_series_d7_with_data() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org D7").await;
    let agent_id = seed_agent(&db, org_id, "agent_d7_1").await;

    // Seed control records
    seed_control_record(&db, agent_id, org_id, "AB-001-AB", "valid").await;
    seed_control_record(&db, agent_id, org_id, "CD-002-CD", "valid").await;
    seed_control_record(&db, agent_id, org_id, "EF-003-EF", "critical").await;

    let result = stats_queries::get_control_activity_series_query(&db, DashboardRange::D7).await;

    assert!(
        result.is_ok(),
        "get_control_activity_series_query should succeed"
    );
    let series = result.unwrap();

    // D7 should return 7 data points (one per day)
    assert_eq!(series.len(), 7, "D7 range should return 7 daily buckets");

    // At least one bucket should have data
    let total_count: i64 = series.iter().map(|p| p.count).sum();
    assert_eq!(total_count, 3, "Should have 3 control records in total");
}

#[tokio::test]
async fn test_get_control_activity_series_d30_with_data() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org D30").await;
    let agent_id = seed_agent(&db, org_id, "agent_d30_1").await;

    // Seed control records
    seed_control_record(&db, agent_id, org_id, "AB-100-AB", "valid").await;
    seed_control_record(&db, agent_id, org_id, "CD-200-CD", "warning").await;

    let result = stats_queries::get_control_activity_series_query(&db, DashboardRange::D30).await;

    assert!(
        result.is_ok(),
        "get_control_activity_series_query should succeed"
    );
    let series = result.unwrap();

    // D30 should return 30 data points (one per day)
    assert_eq!(series.len(), 30, "D30 range should return 30 daily buckets");

    // Should have at least 2 records
    let total_count: i64 = series.iter().map(|p| p.count).sum();
    assert_eq!(total_count, 2, "Should have 2 control records in total");
}

// ─────────────────────────────────────────────────────────────────────────────
// get_top_agents_query Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_top_agents_with_controls() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Agents").await;
    let agent1_id = seed_agent(&db, org_id, "top_agent_1").await;
    let agent2_id = seed_agent(&db, org_id, "top_agent_2").await;

    // Agent 1 has more controls
    seed_control_record(&db, agent1_id, org_id, "PLATE-A1-001", "valid").await;
    seed_control_record(&db, agent1_id, org_id, "PLATE-A1-002", "valid").await;
    seed_control_record(&db, agent1_id, org_id, "PLATE-A1-003", "valid").await;

    // Agent 2 has fewer controls
    seed_control_record(&db, agent2_id, org_id, "PLATE-A2-001", "valid").await;

    let result = stats_queries::get_top_agents_query(&db, DashboardRange::H24, 10).await;

    assert!(result.is_ok(), "get_top_agents_query should succeed");
    let agents = result.unwrap();

    assert_eq!(agents.len(), 2, "Should return 2 agents");
    // Should be ordered by controls_count DESC
    assert_eq!(
        agents[0].controls_count, 3,
        "Agent 1 should have 3 controls"
    );
    assert_eq!(agents[1].controls_count, 1, "Agent 2 should have 1 control");
}

#[tokio::test]
async fn test_get_top_agents_limit() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Limit").await;

    // Create 5 agents with controls
    for i in 1..=5 {
        let agent_id = seed_agent(&db, org_id, &format!("limit_agent_{}", i)).await;
        seed_control_record(&db, agent_id, org_id, &format!("PLATE-L{}", i), "valid").await;
    }

    // Request only top 3
    let result = stats_queries::get_top_agents_query(&db, DashboardRange::H24, 3).await;

    assert!(result.is_ok(), "get_top_agents_query should succeed");
    let agents = result.unwrap();

    assert_eq!(agents.len(), 3, "Should return only 3 agents due to LIMIT");
}

#[tokio::test]
async fn test_get_top_agents_empty() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    // No agents or controls
    let result = stats_queries::get_top_agents_query(&db, DashboardRange::H24, 10).await;

    assert!(
        result.is_ok(),
        "get_top_agents_query should succeed with no data"
    );
    let agents = result.unwrap();

    assert!(
        agents.is_empty(),
        "Should return empty vector when no agents exist"
    );
}

#[tokio::test]
async fn test_get_top_agents_with_online_status() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Online").await;
    let agent_id = seed_agent(&db, org_id, "online_agent").await;

    seed_control_record(&db, agent_id, org_id, "PLATE-ONLINE", "valid").await;

    // Seed a recent location (agent is online)
    seed_agent_location(&db, agent_id, 4.0511, 9.7679).await;

    let result = stats_queries::get_top_agents_query(&db, DashboardRange::H24, 10).await;

    assert!(result.is_ok(), "get_top_agents_query should succeed");
    let agents = result.unwrap();

    assert_eq!(agents.len(), 1, "Should return 1 agent");
    assert!(
        agents[0].is_online,
        "Agent with recent location should be marked online"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// get_activity_feed_query Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_activity_feed_with_records() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Feed").await;
    let agent_id = seed_agent(&db, org_id, "feed_agent").await;

    // Seed multiple control records
    let _id1 = seed_control_record(&db, agent_id, org_id, "FEED-001", "valid").await;
    let _id2 = seed_control_record(&db, agent_id, org_id, "FEED-002", "warning").await;
    let _id3 = seed_control_record(&db, agent_id, org_id, "FEED-003", "critical").await;

    let result = stats_queries::get_activity_feed_query(&db, 10).await;

    assert!(result.is_ok(), "get_activity_feed_query should succeed");
    let feed = result.unwrap();

    assert_eq!(feed.len(), 3, "Should return 3 feed items");
    // Should be ordered by created_at DESC (most recent first)
    assert_eq!(feed[0].plate_number, "FEED-003");
    assert_eq!(feed[0].overall_status, "critical");
}

#[tokio::test]
async fn test_get_activity_feed_with_limit() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Feed Limit").await;
    let agent_id = seed_agent(&db, org_id, "feed_limit_agent").await;

    // Seed 5 control records
    for i in 1..=5 {
        seed_control_record(&db, agent_id, org_id, &format!("LIMIT-{}", i), "valid").await;
    }

    let result = stats_queries::get_activity_feed_query(&db, 3).await;

    assert!(result.is_ok(), "get_activity_feed_query should succeed");
    let feed = result.unwrap();

    assert_eq!(feed.len(), 3, "Should return only 3 items due to LIMIT");
}

#[tokio::test]
async fn test_get_activity_feed_empty() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    // No control records
    let result = stats_queries::get_activity_feed_query(&db, 10).await;

    assert!(
        result.is_ok(),
        "get_activity_feed_query should succeed with no data"
    );
    let feed = result.unwrap();

    assert!(
        feed.is_empty(),
        "Should return empty vector when no records exist"
    );
}

#[tokio::test]
async fn test_get_activity_feed_excludes_deleted() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Feed Deleted").await;
    let agent_id = seed_agent(&db, org_id, "feed_deleted_agent").await;

    // Seed a valid record
    seed_control_record(&db, agent_id, org_id, "VALID-001", "valid").await;

    // Seed a deleted record
    let deleted_id = seed_control_record(&db, agent_id, org_id, "DELETED-001", "valid").await;
    sqlx::query(r#"UPDATE control_records SET deleted_at = NOW() WHERE id = $1"#)
        .bind(deleted_id)
        .execute(&db)
        .await
        .expect("Failed to soft-delete record");

    let result = stats_queries::get_activity_feed_query(&db, 10).await;

    assert!(result.is_ok(), "get_activity_feed_query should succeed");
    let feed = result.unwrap();

    assert_eq!(feed.len(), 1, "Should exclude deleted records");
    assert_eq!(feed[0].plate_number, "VALID-001");
}

// ─────────────────────────────────────────────────────────────────────────────
// get_recent_alerts_query Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_recent_alerts_with_warnings_and_criticals() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Alerts").await;
    let agent_id = seed_agent(&db, org_id, "alert_agent").await;

    // Seed mixed status records
    seed_control_record(&db, agent_id, org_id, "ALERT-OK", "valid").await;
    seed_control_record(&db, agent_id, org_id, "ALERT-WARN", "warning").await;
    seed_control_record(&db, agent_id, org_id, "ALERT-CRIT", "critical").await;

    let result = stats_queries::get_recent_alerts_query(&db, 10).await;

    assert!(result.is_ok(), "get_recent_alerts_query should succeed");
    let alerts = result.unwrap();

    assert_eq!(
        alerts.len(),
        2,
        "Should return only warning and critical records"
    );
    assert!(
        alerts.iter().any(|a| a.plate_number == "ALERT-WARN"),
        "Should include warning"
    );
    assert!(
        alerts.iter().any(|a| a.plate_number == "ALERT-CRIT"),
        "Should include critical"
    );
    assert!(
        alerts.iter().all(|a| a.plate_number != "ALERT-OK"),
        "Should exclude ok records"
    );
}

#[tokio::test]
async fn test_get_recent_alerts_only_ok_records() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org No Alerts").await;
    let agent_id = seed_agent(&db, org_id, "no_alert_agent").await;

    // Seed only valid records
    seed_control_record(&db, agent_id, org_id, "OK-001", "valid").await;
    seed_control_record(&db, agent_id, org_id, "OK-002", "valid").await;

    let result = stats_queries::get_recent_alerts_query(&db, 10).await;

    assert!(result.is_ok(), "get_recent_alerts_query should succeed");
    let alerts = result.unwrap();

    assert!(
        alerts.is_empty(),
        "Should return empty when no warning/critical records exist"
    );
}

#[tokio::test]
async fn test_get_recent_alerts_with_limit() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Alert Limit").await;
    let agent_id = seed_agent(&db, org_id, "alert_limit_agent").await;

    // Seed 5 critical records
    for i in 1..=5 {
        seed_control_record(&db, agent_id, org_id, &format!("CRIT-{}", i), "critical").await;
    }

    let result = stats_queries::get_recent_alerts_query(&db, 2).await;

    assert!(result.is_ok(), "get_recent_alerts_query should succeed");
    let alerts = result.unwrap();

    assert_eq!(alerts.len(), 2, "Should return only 2 items due to LIMIT");
}

#[tokio::test]
async fn test_get_recent_alerts_includes_address() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Test Org Alert Address").await;
    let agent_id = seed_agent(&db, org_id, "alert_address_agent").await;

    seed_control_record(&db, agent_id, org_id, "ALERT-ADDR", "critical").await;

    let result = stats_queries::get_recent_alerts_query(&db, 10).await;

    assert!(result.is_ok(), "get_recent_alerts_query should succeed");
    let alerts = result.unwrap();

    assert_eq!(alerts.len(), 1);
    assert!(
        alerts[0].address.is_some(),
        "Alert should have address field"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// get_dashboard_stats_query Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_dashboard_stats_complete() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Dashboard Test Org").await;
    let agent_id = seed_agent(&db, org_id, "dashboard_agent").await;

    // Seed vehicles
    seed_vehicle(&db, "DASH-001").await;
    seed_vehicle(&db, "DASH-002").await;

    // Seed control records (today's)
    seed_control_record(&db, agent_id, org_id, "CTRL-001", "valid").await;
    seed_control_record(&db, agent_id, org_id, "CTRL-002", "critical").await;

    // Seed pending submission
    seed_pending_submission(&db, agent_id, "PENDING-PLATE", "pending").await;

    // Seed agent location
    seed_agent_location(&db, agent_id, 4.0511, 9.7679).await;

    let result = stats_queries::get_dashboard_stats_query(&db).await;

    assert!(result.is_ok(), "get_dashboard_stats_query should succeed");
    let stats = result.unwrap();

    // Verify counts
    assert_eq!(stats.today_controls, 2, "Should have 2 controls today");
    assert_eq!(
        stats.active_alerts, 1,
        "Should have 1 active alert (critical)"
    );
    assert_eq!(stats.total_vehicles, 2, "Should have 2 vehicles");
    assert_eq!(stats.online_agents, 1, "Should have 1 active agent");
    assert_eq!(
        stats.pending_submissions, 1,
        "Should have 1 pending submission"
    );
    assert_eq!(stats.organizations_count, 1, "Should have 1 organization");

    // Verify activity_24h has 24 buckets
    assert_eq!(
        stats.activity_24h.len(),
        24,
        "activity_24h should have 24 hourly buckets"
    );

    // Verify live_agents has 1 agent
    assert_eq!(stats.live_agents.len(), 1, "Should have 1 live agent");
    assert_eq!(stats.live_agents[0].agent_id, agent_id);
}

#[tokio::test]
async fn test_get_dashboard_stats_empty_database() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    // No data at all
    let result = stats_queries::get_dashboard_stats_query(&db).await;

    assert!(
        result.is_ok(),
        "get_dashboard_stats_query should succeed with no data"
    );
    let stats = result.unwrap();

    // All counts should be zero
    assert_eq!(stats.today_controls, 0);
    assert_eq!(stats.active_alerts, 0);
    assert_eq!(stats.total_vehicles, 0);
    assert_eq!(stats.online_agents, 0);
    assert_eq!(stats.pending_submissions, 0);
    assert_eq!(stats.organizations_count, 0);

    // Activity should still have 24 empty buckets
    assert_eq!(stats.activity_24h.len(), 24);
    for activity in &stats.activity_24h {
        assert_eq!(activity.count, 0);
    }

    // No live agents
    assert!(stats.live_agents.is_empty());
}

#[tokio::test]
async fn test_get_dashboard_stats_activity_24h_structure() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Activity Test Org").await;
    let agent_id = seed_agent(&db, org_id, "activity_agent").await;

    // Seed 1 control record
    seed_control_record(&db, agent_id, org_id, "ACT-001", "valid").await;

    let result = stats_queries::get_dashboard_stats_query(&db).await;

    assert!(result.is_ok(), "get_dashboard_stats_query should succeed");
    let stats = result.unwrap();

    // Verify activity_24h structure
    assert_eq!(stats.activity_24h.len(), 24);
    for activity in &stats.activity_24h {
        // Hour labels should be formatted as HH:00
        assert!(
            activity.hour.contains(':'),
            "Hour label should contain colon separator"
        );
    }

    // Total count across all buckets should be 1
    let total_count: i64 = stats.activity_24h.iter().map(|a| a.count).sum();
    assert_eq!(total_count, 1, "Total count across all buckets should be 1");
}

#[tokio::test]
async fn test_get_dashboard_stats_live_agents_location() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    let org_id = seed_organization(&db, "Live Agent Test Org").await;
    let agent1_id = seed_agent(&db, org_id, "live_agent_1").await;
    let _agent2_id = seed_agent(&db, org_id, "live_agent_2").await;

    // Only agent 1 has a recent location
    seed_agent_location(&db, agent1_id, 4.0511, 9.7679).await;

    let result = stats_queries::get_dashboard_stats_query(&db).await;

    assert!(result.is_ok(), "get_dashboard_stats_query should succeed");
    let stats = result.unwrap();

    // Only agent 1 should be in live_agents (recent location)
    assert_eq!(stats.live_agents.len(), 1);
    assert_eq!(stats.live_agents[0].agent_id, agent1_id);
    assert_eq!(stats.live_agents[0].latitude, 4.0511);
    assert_eq!(stats.live_agents[0].longitude, 9.7679);
}

#[tokio::test]
async fn test_get_dashboard_stats_multiple_organizations() {
    let (db, _pg, _redis) = setup_test_infrastructure().await;

    // Create multiple organizations
    let org1_id = seed_organization(&db, "Org Alpha").await;
    let org2_id = seed_organization(&db, "Org Beta").await;

    // Create agents in both orgs
    let agent1_id = seed_agent(&db, org1_id, "multi_org_agent_1").await;
    let agent2_id = seed_agent(&db, org2_id, "multi_org_agent_2").await;

    // Create control records
    seed_control_record(&db, agent1_id, org1_id, "MULTI-001", "valid").await;
    seed_control_record(&db, agent2_id, org2_id, "MULTI-002", "critical").await;

    let result = stats_queries::get_dashboard_stats_query(&db).await;

    assert!(result.is_ok(), "get_dashboard_stats_query should succeed");
    let stats = result.unwrap();

    assert_eq!(stats.organizations_count, 2, "Should count 2 organizations");
    assert_eq!(stats.today_controls, 2, "Should have 2 controls today");
    assert_eq!(stats.active_alerts, 1, "Should have 1 critical alert");
    // Online agents should be 2 (both are ACTIVE status)
    assert_eq!(stats.online_agents, 2);
}
