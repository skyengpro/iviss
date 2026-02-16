use crate::dto::common::{IdentificationMode, Status};
use crate::dto::list_control::{
    ActionType, ControlAction, ControlLocation, ControlResults, ListControlResponse,
};
use crate::errors::AppError;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

pub async fn get_control_records(
    pool: &PgPool,
    start_date: Option<String>,
    end_date: Option<String>,
    agent_id: Option<Uuid>,
    status: Option<Status>,
    plate: Option<String>,
) -> Result<Vec<ListControlResponse>, AppError> {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT 
            c.id,
            c.plate_number,
            c.agent_id,
            u.full_name as agent_name,
            c.organization_id,
            c.timestamp,
            c.identification_mode,
            c.ocr_confidence,
            c.overall_status,
            c.latitude,
            c.longitude,
            c.address,
            c.results_json,
            c.notes
        FROM control_records c
        JOIN users u ON c.agent_id = u.id
        WHERE c.deleted_at IS NULL
        "#,
    );

    if let Some(start) = start_date {
        // Assuming ISO string, cast to timestamp
        query_builder.push(" AND c.timestamp >= ");
        query_builder.push_bind(start).push("::timestamp");
    }

    if let Some(end) = end_date {
        query_builder.push(" AND c.timestamp <= ");
        query_builder.push_bind(end).push("::timestamp");
    }

    if let Some(aid) = agent_id {
        query_builder.push(" AND c.agent_id = ");
        query_builder.push_bind(aid);
    }

    if let Some(s) = status {
        // Map Status enum to string for DB query if needed, or use as is if DB type matches
        // For now, let's assume it maps to the string value in DB
        let status_str = match s {
            Status::Valid => "valid",
            Status::Warning => "warning",
            Status::Critical => "critical",
            Status::Pending => "pending",
        };
        query_builder.push(" AND c.overall_status = ");
        query_builder.push_bind(status_str);
    }

    if let Some(p) = plate {
        query_builder.push(" AND c.plate_number ILIKE ");
        query_builder.push_bind(format!("%{}%", p));
    }

    query_builder.push(" ORDER BY c.timestamp DESC LIMIT 100");

    let rows = query_builder
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let mut responses = Vec::new();

    for row in rows {
        let id: Uuid = row.get("id");

        // Fetch actions for this control record
        // N+1 query problem here, but for now it's acceptable for MVP.
        // Can be optimized with a join or separate batch query later.
        let actions = get_actions_for_control(pool, id).await?;

        // Parse results_json
        let results_json: serde_json::Value = row.get("results_json");
        let results = serde_json::from_value(results_json).unwrap_or(ControlResults {
            registration: Status::Valid, // Default fallback
            insurance: Status::Valid,
            technical_inspection: Status::Valid,
            wanted_status: Status::Valid,
            customs_status: Status::Valid,
        });

        // Determine identification mode from string
        let id_mode_str: String = row.get("identification_mode");
        let identification_mode = match id_mode_str.as_str() {
            "manual" => IdentificationMode::Manual,
            "photo" => IdentificationMode::Photo,
            "live" => IdentificationMode::Live,
            _ => IdentificationMode::Manual,
        };

        // Determine overall status
        let status_str: String = row.get("overall_status");
        let status = match status_str.as_str() {
            "valid" => Status::Valid,
            "warning" => Status::Warning,
            "critical" => Status::Critical,
            _ => Status::Valid,
        };

        responses.push(ListControlResponse {
            id,
            plate_number: row.get("plate_number"),
            agent_name: row.get("agent_name"),
            agent_id: row.get("agent_id"),
            organization_id: row.get("organization_id"),
            timestamp: row
                .get::<time::PrimitiveDateTime, _>("timestamp")
                .to_string(), // Simplified date handling
            status,
            identification_mode,
            confidence: row.get("ocr_confidence"), // Integer in DB, float in DTO? Check schema
            location: ControlLocation {
                address: row.get("address"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
            },
            results,
            actions,
            notes: row.get("notes"),
        });
    }

    Ok(responses)
}

async fn get_actions_for_control(
    pool: &PgPool,
    control_id: Uuid,
) -> Result<Vec<ControlAction>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT action_type, description, timestamp
        FROM control_actions
        WHERE control_id = $1
        ORDER BY timestamp ASC
        "#,
    )
    .bind(control_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut actions = Vec::new();
    for row in rows {
        let action_type_str: String = row.get("action_type");
        let action_type = match action_type_str.as_str() {
            "citation" => ActionType::Citation,
            "impound" => ActionType::Impound,
            "flag" => ActionType::Flag,
            "warning" => ActionType::Check, // Mapping 'warning' to 'Check' or maybe we need 'Warning' in enum?
            "check" => ActionType::Check,
            "release" => ActionType::Release,
            _ => ActionType::Check,
        };

        actions.push(ControlAction {
            action_type,
            description: row.get("description"),
            timestamp: row
                .get::<time::PrimitiveDateTime, _>("timestamp")
                .to_string(),
        });
    }

    Ok(actions)
}
