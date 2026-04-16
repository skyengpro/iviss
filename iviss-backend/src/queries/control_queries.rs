use crate::dto::common::{IdentificationMode, Status};
use crate::dto::create_control::CreateControlRequest;
use crate::dto::list_control::{
    ActionType, ControlAction, ControlLocation, ControlPagedQuery, ControlResults,
    ListControlResponse,
};
use crate::errors::AppError;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

pub async fn create_control_record(
    pool: &PgPool,
    req: CreateControlRequest,
) -> Result<Uuid, AppError> {
    let control_id = Uuid::new_v4();
    let current_time = time::OffsetDateTime::now_utc();

    // Determine overall status based on results
    let status_str = if req.results.wanted_status == Status::Critical
        || req.results.insurance == Status::Critical
    {
        "critical"
    } else if req.results.technical_inspection == Status::Warning
        || req.results.customs_status == Status::Warning
    {
        "warning"
    } else {
        "valid"
    };

    let id_mode_str = match req.identification_mode {
        IdentificationMode::Manual => "manual",
        IdentificationMode::Photo => "photo",
        IdentificationMode::Live => "live",
    };

    // 1. Insert Control Record
    sqlx::query(
        r#"
        INSERT INTO control_records (
            id, plate_number, agent_id, organization_id, timestamp,
            latitude, longitude, address, identification_mode, ocr_confidence,
            overall_status, results_json, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(control_id)
    .bind(req.plate_number)
    .bind(req.agent_id)
    .bind(req.organization_id)
    .bind(current_time)
    .bind(req.latitude)
    .bind(req.longitude)
    .bind(req.address)
    .bind(id_mode_str)
    .bind(req.ocr_confidence)
    .bind(status_str)
    .bind(serde_json::to_value(&req.results).unwrap_or(serde_json::json!({})))
    .bind(req.notes)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    // 2. Insert Initial Action (Log "Check")
    sqlx::query(
        r#"
        INSERT INTO control_actions (
            id, control_id, action_type, description, timestamp
        )
        VALUES ($1, $2, 'flag', 'Control performed via mobile app', $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(control_id)
    .bind(current_time)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(control_id)
}

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
            c.ocr_confidence::DOUBLE PRECISION as ocr_confidence,
            c.overall_status,
            c.latitude::DOUBLE PRECISION as latitude,
            c.longitude::DOUBLE PRECISION as longitude,
            c.address,
            c.results_json,
            c.notes,
            v.brand,
            v.model,
            v.year,
            v.color,
            v.engine_power,
            v.fuel_type,
            v.chassis_number,
            vo.name as owner_name,
            vo.address as owner_address,
            vo.national_id as owner_national_id
        FROM control_records c
        JOIN users u ON c.agent_id = u.id
        LEFT JOIN vehicles v ON c.plate_number = v.plate_number
        LEFT JOIN vehicle_owners vo ON v.id = vo.vehicle_id AND vo.is_current_owner = TRUE
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
        query_builder.push_bind(format!("%{p}%"));
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
        let results_json: Option<serde_json::Value> = row.get("results_json");
        let results = results_json
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(ControlResults {
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
            "pending" => Status::Pending,
            _ => Status::Valid,
        };

        responses.push(ListControlResponse {
            id,
            plate_number: row.get("plate_number"),
            agent_name: row.get("agent_name"),
            agent_id: row.get("agent_id"),
            organization_id: row.get("organization_id"),
            timestamp: row
                .get::<time::OffsetDateTime, _>("timestamp")
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
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
            vehicle: row.get::<Option<String>, _>("brand").map(|brand| {
                use crate::dto::search_vehicle::{OwnerInfo, VehicleInfo};
                VehicleInfo {
                    brand,
                    model: row.get::<Option<String>, _>("model").unwrap_or_default(),
                    year: row.get::<Option<i32>, _>("year").unwrap_or_default(),
                    color: row.get("color"),
                    engine_power: row.get("engine_power"),
                    fuel_type: row.get("fuel_type"),
                    chassis_number: row
                        .get::<Option<String>, _>("chassis_number")
                        .unwrap_or_default(),
                    owner: OwnerInfo {
                        name: row
                            .get::<Option<String>, _>("owner_name")
                            .unwrap_or_default(),
                        address: row.get("owner_address"),
                        national_id: row.get("owner_national_id"),
                    },
                }
            }),
        });
    }

    Ok(responses)
}

pub async fn get_paged_control_records(
    pool: &PgPool,
    query: &ControlPagedQuery,
    page: i64,
    page_size: i64,
) -> Result<(Vec<ListControlResponse>, i64), AppError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    fn apply_filters(qb: &mut QueryBuilder<Postgres>, query: &ControlPagedQuery) {
        if let Some(start) = query.start_date.as_ref() {
            qb.push(" AND c.timestamp >= ");
            qb.push_bind(start.clone()).push("::timestamp");
        }

        if let Some(end) = query.end_date.as_ref() {
            qb.push(" AND c.timestamp <= ");
            qb.push_bind(end.clone()).push("::timestamp");
        }

        if let Some(aid) = query.agent_id {
            qb.push(" AND c.agent_id = ");
            qb.push_bind(aid);
        }

        if let Some(oid) = query.organization_id {
            qb.push(" AND c.organization_id = ");
            qb.push_bind(oid);
        }

        if let Some(s) = query.status.as_ref() {
            let status_str = match s {
                Status::Valid => "valid",
                Status::Warning => "warning",
                Status::Critical => "critical",
                Status::Pending => "pending",
            };
            qb.push(" AND c.overall_status = ");
            qb.push_bind(status_str);
        }

        if let Some(p) = query.plate.as_ref() {
            qb.push(" AND c.plate_number ILIKE ");
            qb.push_bind(format!("%{p}%"));
        }

        if let Some(text) = query.q.as_ref() {
            let like = format!("%{text}%");
            qb.push(" AND (");
            qb.push("c.plate_number ILIKE ");
            qb.push_bind(like.clone());
            qb.push(" OR u.full_name ILIKE ");
            qb.push_bind(like.clone());
            qb.push(" OR c.address ILIKE ");
            qb.push_bind(like);
            qb.push(")");
        }
    }

    // ---- Count query ----
    let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT COUNT(*) as total
        FROM control_records c
        JOIN users u ON c.agent_id = u.id
        WHERE c.deleted_at IS NULL
        "#,
    );
    apply_filters(&mut count_qb, query);

    let total: i64 = count_qb
        .build()
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?
        .get("total");

    // ---- Items query ----
    let mut items_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT
            c.id,
            c.plate_number,
            c.agent_id,
            u.full_name as agent_name,
            c.organization_id,
            c.timestamp,
            c.identification_mode,
            c.ocr_confidence::DOUBLE PRECISION as ocr_confidence,
            c.overall_status,
            c.latitude::DOUBLE PRECISION as latitude,
            c.longitude::DOUBLE PRECISION as longitude,
            c.address,
            c.results_json,
            c.notes,
            v.brand,
            v.model,
            v.year,
            v.color,
            v.engine_power,
            v.fuel_type,
            v.chassis_number,
            vo.name as owner_name,
            vo.address as owner_address,
            vo.national_id as owner_national_id
        FROM control_records c
        JOIN users u ON c.agent_id = u.id
        LEFT JOIN vehicles v ON c.plate_number = v.plate_number
        LEFT JOIN vehicle_owners vo ON v.id = vo.vehicle_id AND vo.is_current_owner = TRUE
        WHERE c.deleted_at IS NULL
        "#,
    );
    apply_filters(&mut items_qb, query);
    items_qb.push(" ORDER BY c.timestamp DESC ");
    items_qb.push(" LIMIT ").push_bind(page_size);
    items_qb.push(" OFFSET ").push_bind(offset);

    let rows = items_qb
        .build()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    let mut responses = Vec::new();

    for row in rows {
        let id: Uuid = row.get("id");
        let actions = get_actions_for_control(pool, id).await?;

        let results_json: Option<serde_json::Value> = row.get("results_json");
        let results = results_json
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(ControlResults {
                registration: Status::Valid,
                insurance: Status::Valid,
                technical_inspection: Status::Valid,
                wanted_status: Status::Valid,
                customs_status: Status::Valid,
            });

        let id_mode_str: String = row.get("identification_mode");
        let identification_mode = match id_mode_str.as_str() {
            "manual" => IdentificationMode::Manual,
            "photo" => IdentificationMode::Photo,
            "live" => IdentificationMode::Live,
            _ => IdentificationMode::Manual,
        };

        let status_str: String = row.get("overall_status");
        let status = match status_str.as_str() {
            "valid" => Status::Valid,
            "warning" => Status::Warning,
            "critical" => Status::Critical,
            "pending" => Status::Pending,
            _ => Status::Valid,
        };

        responses.push(ListControlResponse {
            id,
            plate_number: row.get("plate_number"),
            agent_name: row.get("agent_name"),
            agent_id: row.get("agent_id"),
            organization_id: row.get("organization_id"),
            timestamp: row
                .get::<time::OffsetDateTime, _>("timestamp")
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            status,
            identification_mode,
            confidence: row.get("ocr_confidence"),
            location: ControlLocation {
                address: row.get("address"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
            },
            results,
            actions,
            notes: row.get("notes"),
            vehicle: row.get::<Option<String>, _>("brand").map(|brand| {
                use crate::dto::search_vehicle::{OwnerInfo, VehicleInfo};
                VehicleInfo {
                    brand,
                    model: row.get::<Option<String>, _>("model").unwrap_or_default(),
                    year: row.get::<Option<i32>, _>("year").unwrap_or_default(),
                    color: row.get("color"),
                    engine_power: row.get("engine_power"),
                    fuel_type: row.get("fuel_type"),
                    chassis_number: row
                        .get::<Option<String>, _>("chassis_number")
                        .unwrap_or_default(),
                    owner: OwnerInfo {
                        name: row
                            .get::<Option<String>, _>("owner_name")
                            .unwrap_or_default(),
                        address: row.get("owner_address"),
                        national_id: row.get("owner_national_id"),
                    },
                }
            }),
        });
    }

    Ok((responses, total))
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
                .get::<time::OffsetDateTime, _>("timestamp")
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        });
    }

    Ok(actions)
}
