use crate::dto::organizations::{CreateOrganizationRequest, Organization, OrganizationDetails, OrganizationType, UpdateOrganizationRequest};
use crate::errors::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn list_organizations(pool: &PgPool) -> Result<Vec<Organization>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, type, region
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

    // Convert enum to string for database
    let type_str = match req.org_type {
        OrganizationType::Police => "police",
        OrganizationType::Customs => "customs",
        OrganizationType::BorderControl => "border_control",
        OrganizationType::Other => "other",
    };

    let row = sqlx::query(
        r#"
        INSERT INTO organizations (name, type, region)
        VALUES ($1, $2, $3)
        RETURNING id, name, type, region
        "#,
    )
    .bind(&req.name)
    .bind(type_str)
    .bind(&req.region)
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
    })
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
    })
}

pub async fn update_organization(
    pool: &PgPool,
    id: Uuid,
    req: UpdateOrganizationRequest,
) -> Result<Organization, AppError> {
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

    // Build dynamic update query
    let mut query = String::from("UPDATE organizations SET updated_at = NOW()");
    let mut param_count = 1;

    if req.name.is_some() {
        query.push_str(&format!(", name = ${}", param_count));
        param_count += 1;
    }

    if req.org_type.is_some() {
        query.push_str(&format!(", type = ${}", param_count));
        param_count += 1;
    }

    if req.region.is_some() {
        query.push_str(&format!(", region = ${}", param_count));
        param_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING id, name, type, region", param_count));

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
        return Err(AppError::bad_request(&format!(
            "Cannot delete organization with {} active users",
            user_count
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
