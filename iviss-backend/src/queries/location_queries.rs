use crate::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn update_agent_location_query(
    pool: &PgPool,
    agent_id: Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO agent_locations (agent_id, latitude, longitude, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (agent_id)
        DO UPDATE SET
            latitude = EXCLUDED.latitude,
            longitude = EXCLUDED.longitude,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(agent_id)
    .bind(latitude)
    .bind(longitude)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    async fn setup_test_db() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
        let postgres = Postgres::default().start().await.unwrap();
        let pg_host = postgres.get_host().await.unwrap();
        let pg_port = postgres.get_host_port_ipv4(5432).await.unwrap();
        let db_url = format!("postgres://postgres:postgres@{pg_host}:{pg_port}/postgres");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        (pool, postgres)
    }

    /// Helper to create a test user with organization for FK constraints
    async fn create_test_user(pool: &PgPool) -> Uuid {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create organization first
        sqlx::query(r#"INSERT INTO organizations (id, name, type) VALUES ($1, $2, $3)"#)
            .bind(org_id)
            .bind("Test Organization")
            .bind("police")
            .execute(pool)
            .await
            .unwrap();

        // Create user
        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, password_hash, phone_number, 
                role, status, full_name, created_at, organization_id, badge_id
            )
            VALUES ($1, $2, $3, $4, $5, $6::user_role, $7::user_status, $8, NOW(), $9, $10)
            "#,
        )
        .bind(user_id)
        .bind(format!(
            "user_{}",
            user_id.to_string().split('-').next().unwrap()
        ))
        .bind(format!(
            "user{}@test.com",
            user_id.to_string().split('-').next().unwrap()
        ))
        .bind(None::<String>) // Agents don't need password_hash
        .bind(format!("+{:012}", user_id.as_u128() % 1000000000000))
        .bind("agent")
        .bind("ACTIVE")
        .bind("Test Agent")
        .bind(org_id)
        .bind(Some(format!(
            "BADGE-{}",
            user_id.to_string().split('-').next().unwrap()
        )))
        .execute(pool)
        .await
        .unwrap();

        user_id
    }

    #[tokio::test]
    async fn test_update_agent_location_inserts_new_location() {
        let (pool, _postgres) = setup_test_db().await;
        let agent_id = create_test_user(&pool).await;

        let result = update_agent_location_query(&pool, agent_id, 4.0511, 9.7679).await;
        assert!(result.is_ok(), "Should successfully insert new location");

        // Verify the data was inserted
        let row: (f64, f64) =
            sqlx::query_as("SELECT latitude, longitude FROM agent_locations WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, 4.0511);
        assert_eq!(row.1, 9.7679);
    }

    #[tokio::test]
    async fn test_update_agent_location_updates_existing_location() {
        let (pool, _postgres) = setup_test_db().await;
        let agent_id = create_test_user(&pool).await;

        // First insert
        update_agent_location_query(&pool, agent_id, 4.0511, 9.7679)
            .await
            .unwrap();

        // Second insert (should update)
        update_agent_location_query(&pool, agent_id, 5.1234, 8.5678)
            .await
            .unwrap();

        // Verify the data was updated (only one row exists)
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM agent_locations WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(count, 1, "Should still have only one row");

        // Verify the updated coordinates
        let row: (f64, f64) =
            sqlx::query_as("SELECT latitude, longitude FROM agent_locations WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, 5.1234);
        assert_eq!(row.1, 8.5678);
    }

    #[tokio::test]
    async fn test_update_agent_location_handles_boundary_coordinates() {
        let (pool, _postgres) = setup_test_db().await;
        let agent_id = create_test_user(&pool).await;

        // Test with extreme valid coordinates
        let result = update_agent_location_query(&pool, agent_id, 90.0, 180.0).await;
        assert!(result.is_ok(), "Should handle maximum positive coordinates");

        let row: (f64, f64) =
            sqlx::query_as("SELECT latitude, longitude FROM agent_locations WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, 90.0);
        assert_eq!(row.1, 180.0);
    }

    #[tokio::test]
    async fn test_update_agent_location_handles_negative_coordinates() {
        let (pool, _postgres) = setup_test_db().await;
        let agent_id = create_test_user(&pool).await;

        // Test with negative coordinates (southern/western hemisphere)
        let result = update_agent_location_query(&pool, agent_id, -33.8688, -151.2093).await;
        assert!(result.is_ok(), "Should handle negative coordinates");

        let row: (f64, f64) =
            sqlx::query_as("SELECT latitude, longitude FROM agent_locations WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, -33.8688);
        assert_eq!(row.1, -151.2093);
    }

    #[tokio::test]
    async fn test_update_agent_location_handles_decimal_precision() {
        let (pool, _postgres) = setup_test_db().await;
        let agent_id = create_test_user(&pool).await;

        // Test with high precision decimal coordinates
        let lat = 4.051056700000001;
        let lon = 9.767870000000002;
        let result = update_agent_location_query(&pool, agent_id, lat, lon).await;
        assert!(result.is_ok(), "Should handle high precision decimals");

        let row: (f64, f64) =
            sqlx::query_as("SELECT latitude, longitude FROM agent_locations WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, lat);
        assert_eq!(row.1, lon);
    }
}
