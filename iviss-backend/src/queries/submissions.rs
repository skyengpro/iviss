use crate::dto::common::SubmissionLocation;
use crate::dto::pending_submission::{
    PendingSubmissionDetail, PendingSubmissionListItem, SubmissionAuditLogEntry, SubmissionStatus,
    VehicleDataEntry,
};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ── Create ────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn create_pending_submission(
    pool: &PgPool,
    agent_id: Uuid,
    plate_number: String,
    front_image_url: String,
    back_image_url: String,
    notes: Option<String>,
    location: SubmissionLocation,
) -> Result<Uuid, AppError> {
    let latitude: Option<f64> = location.latitude;
    let longitude: Option<f64> = location.longitude;
    let address: Option<String> = location.address;
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

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn get_pending_submissions(
    pool: &PgPool,
    status_filter: Option<&str>,
) -> Result<Vec<PendingSubmissionListItem>, AppError> {
    let query = match status_filter {
        Some(_) => {
            r#"
            SELECT s.id, s.plate_number, s.status, s.created_at,
                   u.full_name as agent_name
            FROM pending_submissions s
            LEFT JOIN users u ON s.agent_id = u.id
            WHERE s.status = $1
            ORDER BY s.created_at DESC
            "#
        }
        None => {
            r#"
            SELECT s.id, s.plate_number, s.status, s.created_at,
                   u.full_name as agent_name
            FROM pending_submissions s
            LEFT JOIN users u ON s.agent_id = u.id
            ORDER BY s.created_at DESC
            "#
        }
    };

    let rows = if let Some(status) = status_filter {
        sqlx::query(query)
            .bind(status)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?
    } else {
        sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?
    };

    let mut items = Vec::new();
    for row in rows {
        let status_str: String = row.try_get("status").map_err(AppError::database)?;
        let created_at: time::OffsetDateTime =
            row.try_get("created_at").map_err(AppError::database)?;

        items.push(PendingSubmissionListItem {
            id: row.try_get("id").map_err(AppError::database)?,
            plate_number: row.try_get("plate_number").map_err(AppError::database)?,
            agent_name: row.try_get("agent_name").ok(),
            status: SubmissionStatus::from_db_str(&status_str),
            submitted_at: created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        });
    }
    Ok(items)
}

// ── Detail ────────────────────────────────────────────────────────────────────

pub async fn get_submission_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<PendingSubmissionDetail, AppError> {
    let row = sqlx::query(
        r#"
        SELECT s.id, s.plate_number, s.agent_id, s.front_image_url, s.back_image_url,
               s.notes, s.status, s.created_at, s.latitude, s.longitude, s.address,
               s.reviewed_at, s.reviewed_by, s.rejection_reason, s.vehicle_data,
               u.full_name as agent_name,
               r.full_name as reviewer_name
        FROM pending_submissions s
        LEFT JOIN users u ON s.agent_id = u.id
        LEFT JOIN users r ON s.reviewed_by = r.id
        WHERE s.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("Submission not found"))?;

    let status_str: String = row.try_get("status").map_err(AppError::database)?;
    let created_at: time::OffsetDateTime = row.try_get("created_at").map_err(AppError::database)?;
    let reviewed_at: Option<time::OffsetDateTime> = row.try_get("reviewed_at").ok();
    let vehicle_data_json: Option<serde_json::Value> = row.try_get("vehicle_data").ok();

    let location = {
        let lat: Option<f64> = row.try_get("latitude").ok();
        let lon: Option<f64> = row.try_get("longitude").ok();
        let addr: Option<String> = row.try_get("address").ok();
        if lat.is_some() || lon.is_some() || addr.is_some() {
            Some(crate::dto::common::SubmissionLocation {
                latitude: lat,
                longitude: lon,
                address: addr,
            })
        } else {
            None
        }
    };

    let vehicle_data: Option<VehicleDataEntry> =
        vehicle_data_json.and_then(|v| serde_json::from_value(v).ok());

    Ok(PendingSubmissionDetail {
        id: row.try_get("id").map_err(AppError::database)?,
        plate_number: row.try_get("plate_number").map_err(AppError::database)?,
        agent_id: row.try_get("agent_id").map_err(AppError::database)?,
        agent_name: row.try_get("agent_name").ok(),
        location,
        front_image_url: row.try_get("front_image_url").ok(),
        back_image_url: row.try_get("back_image_url").ok(),
        notes: row.try_get("notes").ok(),
        status: SubmissionStatus::from_db_str(&status_str),
        submitted_at: created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        reviewed_at: reviewed_at.map(|dt| {
            dt.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        }),
        reviewed_by: row.try_get("reviewed_by").ok(),
        reviewer_name: row.try_get("reviewer_name").ok(),
        rejection_reason: row.try_get("rejection_reason").ok(),
        vehicle_data,
    })
}

// ── Approve ───────────────────────────────────────────────────────────────────

/// Approve a pending submission: update status, upsert vehicle + owner, log audit.
/// Returns the vehicle UUID from the main vehicles table.
pub async fn approve_submission(
    pool: &PgPool,
    submission_id: Uuid,
    reviewer_id: Uuid,
    plate_number: &str,
    vehicle_data: &VehicleDataEntry,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    // 1. Ensure the submission is still pending
    let current_status: String =
        sqlx::query_scalar("SELECT status FROM pending_submissions WHERE id = $1 FOR UPDATE")
            .bind(submission_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| AppError::not_found("Submission not found"))?;

    if current_status != "pending" {
        return Err(AppError::bad_request(format!(
            "Submission is already '{current_status}' and cannot be reviewed again"
        )));
    }

    // 2. Serialise vehicle data as JSON
    let vehicle_data_json =
        serde_json::to_value(vehicle_data).map_err(|e| AppError::internal_error(e.to_string()))?;

    // 3. Update submission
    sqlx::query(
        r#"
        UPDATE pending_submissions
        SET status = 'approved',
            reviewed_by = $1,
            reviewed_at = NOW(),
            vehicle_data = $2
        WHERE id = $3
        "#,
    )
    .bind(reviewer_id)
    .bind(&vehicle_data_json)
    .bind(submission_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    // 4. Upsert vehicle
    let vehicle_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO vehicles (plate_number, chassis_number, brand, model, year, color, engine_power, fuel_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (plate_number) DO UPDATE SET
            chassis_number = EXCLUDED.chassis_number,
            brand          = EXCLUDED.brand,
            model          = EXCLUDED.model,
            year           = EXCLUDED.year,
            color          = EXCLUDED.color,
            engine_power   = EXCLUDED.engine_power,
            fuel_type      = EXCLUDED.fuel_type,
            updated_at     = NOW()
        RETURNING id
        "#,
    )
    .bind(plate_number)
    .bind(&vehicle_data.chassis_number)
    .bind(&vehicle_data.brand)
    .bind(&vehicle_data.model)
    .bind(vehicle_data.year)
    .bind(&vehicle_data.color)
    .bind(&vehicle_data.engine_power)
    .bind(&vehicle_data.fuel_type)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::database)?;

    // 5. Upsert vehicle owner — mark existing as not current, insert new
    sqlx::query(
        r#"
        UPDATE vehicle_owners
        SET is_current_owner = FALSE,
            ownership_end_date = CURRENT_DATE,
            updated_at = NOW()
        WHERE vehicle_id = $1 AND is_current_owner = TRUE
        "#,
    )
    .bind(vehicle_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        r#"
        INSERT INTO vehicle_owners (vehicle_id, name, address, national_id, is_current_owner)
        VALUES ($1, $2, $3, $4, TRUE)
        "#,
    )
    .bind(vehicle_id)
    .bind(&vehicle_data.owner_name)
    .bind(&vehicle_data.owner_address)
    .bind(&vehicle_data.owner_national_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    // 6. Audit log
    sqlx::query(
        r#"
        INSERT INTO submission_audit_log (submission_id, action, performed_by, details)
        VALUES ($1, 'approved', $2, $3)
        "#,
    )
    .bind(submission_id)
    .bind(reviewer_id)
    .bind(&vehicle_data_json)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;

    Ok(vehicle_id)
}

// ── Reject ────────────────────────────────────────────────────────────────────

pub async fn reject_submission(
    pool: &PgPool,
    submission_id: Uuid,
    reviewer_id: Uuid,
    reason: &str,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    // 1. Ensure the submission is still pending
    let current_status: String =
        sqlx::query_scalar("SELECT status FROM pending_submissions WHERE id = $1 FOR UPDATE")
            .bind(submission_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::database)?
            .ok_or_else(|| AppError::not_found("Submission not found"))?;

    if current_status != "pending" {
        return Err(AppError::bad_request(format!(
            "Submission is already '{current_status}' and cannot be reviewed again"
        )));
    }

    // 2. Update submission
    sqlx::query(
        r#"
        UPDATE pending_submissions
        SET status = 'rejected',
            reviewed_by = $1,
            reviewed_at = NOW(),
            rejection_reason = $2
        WHERE id = $3
        "#,
    )
    .bind(reviewer_id)
    .bind(reason)
    .bind(submission_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    // 3. Audit log
    sqlx::query(
        r#"
        INSERT INTO submission_audit_log (submission_id, action, performed_by, reason)
        VALUES ($1, 'rejected', $2, $3)
        "#,
    )
    .bind(submission_id)
    .bind(reviewer_id)
    .bind(reason)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;

    Ok(())
}

// ── Audit Log ─────────────────────────────────────────────────────────────────

pub async fn get_submission_audit_log(
    pool: &PgPool,
    submission_id: Uuid,
) -> Result<Vec<SubmissionAuditLogEntry>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT a.id, a.action, a.performed_by, a.reason, a.details, a.created_at,
               u.full_name as performer_name
        FROM submission_audit_log a
        LEFT JOIN users u ON a.performed_by = u.id
        WHERE a.submission_id = $1
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut entries = Vec::new();
    for row in rows {
        let created_at: time::OffsetDateTime =
            row.try_get("created_at").map_err(AppError::database)?;
        entries.push(SubmissionAuditLogEntry {
            id: row.try_get("id").map_err(AppError::database)?,
            action: row.try_get("action").map_err(AppError::database)?,
            performed_by: row.try_get("performed_by").map_err(AppError::database)?,
            performer_name: row.try_get("performer_name").ok(),
            reason: row.try_get("reason").ok(),
            details: row.try_get("details").ok(),
            created_at: created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        });
    }
    Ok(entries)
}

pub async fn resolve_agent_id(pool: &PgPool, requested: Uuid) -> Result<Uuid, AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(requested)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if exists {
        return Ok(requested);
    }

    let first: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users ORDER BY created_at ASC LIMIT 1")
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;

    match first {
        Some(id) => Ok(id),
        None => Err(AppError::not_found("No users found in database")),
    }
}
