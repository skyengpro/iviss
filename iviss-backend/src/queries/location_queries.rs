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
        "#
    )
    .bind(agent_id)
    .bind(latitude)
    .bind(longitude)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(())
}
