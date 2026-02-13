use crate::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn create_pending_submission(
    pool: &PgPool,
    agent_id: Uuid,
    plate_number: String,
    front_image_url: String,
    back_image_url: String,
    notes: Option<String>,
    // Adding location fields to match DTO
    latitude: Option<f64>,
    longitude: Option<f64>,
    address: Option<String>,
) -> Result<Uuid, AppError> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO pending_submissions (
            agent_id, plate_number, front_image_url, back_image_url, notes,
            latitude, longitude, address, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending')
        RETURNING id
        "#,
        agent_id,
        plate_number,
        front_image_url,
        back_image_url,
        notes,
        latitude,
        longitude,
        address
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(rec.id)
}
