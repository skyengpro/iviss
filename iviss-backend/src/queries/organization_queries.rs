use crate::dto::organizations::{
    CreateOrganizationRequest, Organization, OrganizationDetails, OrganizationType,
    UpdateOrganizationRequest,
};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

const DEFAULT_START_WORK_TIME_MINUTES: u32 = 6 * 60;
const DEFAULT_END_WORK_TIME_MINUTES: u32 = 18 * 60;

fn validate_work_time_range(start_work_time: u32, end_work_time: u32) -> Result<(), AppError> {
    if start_work_time > 1439 || end_work_time > 1439 {
        return Err(AppError::bad_request(
            "startWorkTime and endWorkTime must be between 0 and 1439",
        ));
    }

    if start_work_time >= end_work_time {
        return Err(AppError::bad_request(
            "startWorkTime must be less than endWorkTime",
        ));
    }

    Ok(())
}

pub async fn list_organizations(pool: &PgPool) -> Result<Vec<Organization>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, type, region, start_work_time, end_work_time
        FROM organizations
        WHERE deleted_at IS NULL
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let orgs = rows
        .into_iter()
        .map(|row| {
            let type_str: String = row.get("type");
            let org_type = match type_str.as_str() {
                "police" => OrganizationType::Police,
                "customs" => OrganizationType::Customs,
                "border_control" => OrganizationType::BorderControl,
                _ => OrganizationType::Other,
            };

            Organization {
                id: row.get("id"),
                name: row.get("name"),
                org_type,
                region: row.get("region"),
                start_work_time: row.get::<i32, _>("start_work_time") as u32,
                end_work_time: row.get::<i32, _>("end_work_time") as u32,
            }
        })
        .collect();

    Ok(orgs)
}

pub async fn create_organization(
    pool: &PgPool,
    req: CreateOrganizationRequest,
) -> Result<Organization, AppError> {
    // Validate name is not empty
    if req.name.trim().is_empty() {
        return Err(AppError::bad_request("Organization name cannot be empty"));
    }

    // Check if name already exists
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM organizations 
            WHERE LOWER(name) = LOWER($1) 
            AND deleted_at IS NULL
        )
        "#,
    )
    .bind(&req.name)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if exists {
        return Err(AppError::bad_request("Organization name already exists"));
    }

    let start_work_time = req
        .start_work_time
        .unwrap_or(DEFAULT_START_WORK_TIME_MINUTES);
    let end_work_time = req.end_work_time.unwrap_or(DEFAULT_END_WORK_TIME_MINUTES);
    validate_work_time_range(start_work_time, end_work_time)?;

    // Convert enum to string for database
    let type_str = match req.org_type {
        OrganizationType::Police => "police",
        OrganizationType::Customs => "customs",
        OrganizationType::BorderControl => "border_control",
        OrganizationType::Other => "other",
    };

    let row = sqlx::query(
        r#"
        INSERT INTO organizations (name, type, region, start_work_time, end_work_time)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, type, region, start_work_time, end_work_time
        "#,
    )
    .bind(&req.name)
    .bind(type_str)
    .bind(&req.region)
    .bind(start_work_time as i32)
    .bind(end_work_time as i32)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let type_str: String = row.get("type");
    let org_type = match type_str.as_str() {
        "police" => OrganizationType::Police,
        "customs" => OrganizationType::Customs,
        "border_control" => OrganizationType::BorderControl,
        _ => OrganizationType::Other,
    };

    Ok(Organization {
        id: row.get("id"),
        name: row.get("name"),
        org_type,
        region: row.get("region"),
        start_work_time: row.get::<i32, _>("start_work_time") as u32,
        end_work_time: row.get::<i32, _>("end_work_time") as u32,
    })
}

pub async fn get_organization_work_time(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<(u32, u32), AppError> {
    let row = sqlx::query(
        r#"
        SELECT start_work_time, end_work_time
        FROM organizations
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(organization_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("Organization not found"))?;

    Ok((
        row.get::<i32, _>("start_work_time") as u32,
        row.get::<i32, _>("end_work_time") as u32,
    ))
}

pub async fn get_organization_work_time_cached(
    pool: &PgPool,
    cache: &crate::app_cache::AppCache,
    organization_id: Uuid,
) -> Result<(u32, u32), AppError> {
    if let Some((start, end)) = cache.org_work_time.get(&organization_id).await {
        return Ok((start, end));
    }

    let (start, end) = get_organization_work_time(pool, organization_id).await?;
    cache
        .org_work_time
        .insert(organization_id, (start, end))
        .await;
    Ok((start, end))
}

pub async fn load_organizations_work_time_to_cache(
    pool: &PgPool,
    cache: &crate::app_cache::AppCache,
) -> Result<(), AppError> {
    let cache_clone = cache.clone();
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        let rows = sqlx::query(
            r#"
            SELECT id, start_work_time, end_work_time
            FROM organizations
            WHERE deleted_at IS NULL
            LIMIT 100
            "#,
        )
        .fetch_all(&pool_clone)
        .await;

        match rows {
            Ok(rows) => {
                let count = rows.len();

                for row in rows {
                    let org_id: Uuid = row.get("id");
                    let start: u32 = row.get::<i32, _>("start_work_time") as u32;
                    let end: u32 = row.get::<i32, _>("end_work_time") as u32;
                    cache_clone.org_work_time.insert(org_id, (start, end)).await;
                }

                tracing::info!(
                    count,
                    "Loaded organization work time from PostgreSQL to cache (background task completed)"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to load organization work time from PostgreSQL (background task)");
            }
        }
    });

    tracing::info!("Organization work time loading started in background (max 100)");
    Ok(())
}

pub async fn get_organization_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<OrganizationDetails, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            o.id,
            o.name,
            o.type,
            o.region,
            o.created_at,
            o.updated_at,
            o.start_work_time,
            o.end_work_time,
            COUNT(DISTINCT u.id) FILTER (WHERE u.deleted_at IS NULL) as user_count,
            COUNT(DISTINCT u.id) FILTER (WHERE u.role = 'agent' AND u.status = 'ACTIVE' AND u.deleted_at IS NULL) as active_agents,
            COUNT(DISTINCT c.id) as control_count
        FROM organizations o
        LEFT JOIN users u ON u.organization_id = o.id
        LEFT JOIN control_records c ON c.organization_id = o.id
        WHERE o.id = $1 AND o.deleted_at IS NULL
        GROUP BY o.id, o.name, o.type, o.region, o.created_at, o.updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("Organization not found"))?;

    let type_str: String = row.get("type");
    let org_type = match type_str.as_str() {
        "police" => OrganizationType::Police,
        "customs" => OrganizationType::Customs,
        "border_control" => OrganizationType::BorderControl,
        _ => OrganizationType::Other,
    };

    let created_at: time::PrimitiveDateTime = row.get("created_at");
    let updated_at: time::PrimitiveDateTime = row.get("updated_at");

    Ok(OrganizationDetails {
        id: row.get("id"),
        name: row.get("name"),
        org_type,
        region: row.get("region"),
        user_count: row.get("user_count"),
        active_agents: row.get("active_agents"),
        control_count: row.get("control_count"),
        created_at: created_at.to_string(),
        updated_at: updated_at.to_string(),
        shift_start_hour: row.get::<i32, _>("start_work_time") as u32,
        shift_end_hour: row.get::<i32, _>("end_work_time") as u32,
    })
}

pub async fn update_organization(
    pool: &PgPool,
    id: Uuid,
    req: UpdateOrganizationRequest,
) -> Result<Organization, AppError> {
    let current_row = sqlx::query(
        r#"
        SELECT start_work_time, end_work_time
        FROM organizations
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| AppError::not_found("Organization not found"))?;

    // Check if new name conflicts with existing organization
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(AppError::bad_request("Organization name cannot be empty"));
        }

        let name_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM organizations 
                WHERE LOWER(name) = LOWER($1) 
                AND id != $2
                AND deleted_at IS NULL
            )
            "#,
        )
        .bind(name)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

        if name_exists {
            return Err(AppError::bad_request("Organization name already exists"));
        }
    }

    let current_start_work_time = current_row.get::<i32, _>("start_work_time") as u32;
    let current_end_work_time = current_row.get::<i32, _>("end_work_time") as u32;
    let next_start_work_time = req.start_work_time.unwrap_or(current_start_work_time);
    let next_end_work_time = req.end_work_time.unwrap_or(current_end_work_time);

    if req.start_work_time.is_some() || req.end_work_time.is_some() {
        validate_work_time_range(next_start_work_time, next_end_work_time)?;
    }

    // Build dynamic update query
    let mut query = String::from("UPDATE organizations SET updated_at = NOW()");
    let mut param_count = 1;

    if req.name.is_some() {
        query.push_str(&format!(", name = ${param_count}"));
        param_count += 1;
    }

    if req.org_type.is_some() {
        query.push_str(&format!(", type = ${param_count}"));
        param_count += 1;
    }

    if req.region.is_some() {
        query.push_str(&format!(", region = ${param_count}"));
        param_count += 1;
    }

    if req.start_work_time.is_some() || req.end_work_time.is_some() {
        query.push_str(&format!(", start_work_time = ${param_count}"));
        param_count += 1;
        query.push_str(&format!(", end_work_time = ${param_count}"));
        param_count += 1;
    }

    query.push_str(&format!(
        " WHERE id = ${param_count} RETURNING id, name, type, region, start_work_time, end_work_time"
    ));

    let mut query_builder = sqlx::query(&query);

    if let Some(name) = req.name {
        query_builder = query_builder.bind(name);
    }

    if let Some(org_type) = req.org_type {
        let type_str = match org_type {
            OrganizationType::Police => "police",
            OrganizationType::Customs => "customs",
            OrganizationType::BorderControl => "border_control",
            OrganizationType::Other => "other",
        };
        query_builder = query_builder.bind(type_str);
    }

    if let Some(region) = req.region {
        query_builder = query_builder.bind(region);
    }

    if req.start_work_time.is_some() || req.end_work_time.is_some() {
        query_builder = query_builder.bind(next_start_work_time as i32);
        query_builder = query_builder.bind(next_end_work_time as i32);
    }

    query_builder = query_builder.bind(id);

    let row = query_builder
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    let type_str: String = row.get("type");
    let org_type = match type_str.as_str() {
        "police" => OrganizationType::Police,
        "customs" => OrganizationType::Customs,
        "border_control" => OrganizationType::BorderControl,
        _ => OrganizationType::Other,
    };

    Ok(Organization {
        id: row.get("id"),
        name: row.get("name"),
        org_type,
        region: row.get("region"),
        start_work_time: row.get::<i32, _>("start_work_time") as u32,
        end_work_time: row.get::<i32, _>("end_work_time") as u32,
    })
}

pub async fn delete_organization(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    // Check if organization exists
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM organizations 
            WHERE id = $1 AND deleted_at IS NULL
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if !exists {
        return Err(AppError::not_found("Organization not found"));
    }

    // Check if organization has users
    let user_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM users 
        WHERE organization_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if user_count > 0 {
        return Err(AppError::bad_request(format!(
            "Cannot delete organization with {user_count} active users"
        )));
    }

    // Soft delete
    sqlx::query(
        r#"
        UPDATE organizations 
        SET deleted_at = NOW() 
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(())
}
