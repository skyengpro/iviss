use crate::errors::AppError;
use sqlx::{PgPool, Row};
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
    let rec = sqlx::query(
        r#"
        INSERT INTO pending_submissions (
            agent_id, plate_number, front_image_url, back_image_url, notes,
            latitude, longitude, address, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending')
        RETURNING id
        "#,
    )
    .bind(agent_id)
    .bind(plate_number)
    .bind(front_image_url)
    .bind(back_image_url)
    .bind(notes)
    .bind(latitude)
    .bind(longitude)
    .bind(address)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let id: Uuid = rec.try_get("id").map_err(AppError::database)?;
    Ok(id)
}

pub async fn get_pending_submissions(
    pool: &PgPool,
) -> Result<Vec<crate::dto::pending_submission::PendingSubmissionListItem>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT s.id, s.plate_number, s.status, s.created_at as submitted_at, u.name as agent_name
        FROM pending_submissions s
        LEFT JOIN users u ON s.agent_id = u.id
        ORDER BY s.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(crate::dto::pending_submission::PendingSubmissionListItem {
            id: row.try_get("id").map_err(AppError::database)?,
            plate_number: row.try_get("plate_number").map_err(AppError::database)?,
            agent_name: row.try_get("agent_name").map_err(AppError::database).ok(),
            status: crate::dto::pending_submission::SubmissionStatus::Pending, // Simplified for MVP
            submitted_at: "".to_string(),                                      // Simplified
        });
    }
    Ok(items)
}

pub async fn get_submission_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<crate::dto::pending_submission::PendingSubmissionRequest, AppError> {
    let row = sqlx::query(
        r#"
        SELECT s.*, u.name as agent_name
        FROM pending_submissions s
        LEFT JOIN users u ON s.agent_id = u.id
        WHERE s.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    Ok(crate::dto::pending_submission::PendingSubmissionRequest {
        id: row.try_get("id").map_err(AppError::database)?,
        plate_number: row.try_get("plate_number").map_err(AppError::database)?,
        agent_id: row.try_get("agent_id").map_err(AppError::database)?,
        agent_name: row.try_get("agent_name").map_err(AppError::database).ok(),
        location: None,
        front_image_url: row.try_get("front_image_url").map_err(AppError::database)?,
        back_image_url: row.try_get("back_image_url").map_err(AppError::database)?,
        notes: row.try_get("notes").map_err(AppError::database).ok(),
        status: crate::dto::pending_submission::SubmissionStatus::Pending,
        submitted_at: "".to_string(),
        reviewed_at: None,
        reviewed_by: None,
        admin_note: None,
    })
}
