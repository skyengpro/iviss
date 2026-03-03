use crate::dto::organizations::{Organization, OrganizationType};
use crate::errors::AppError;
use sqlx::{PgPool, Row};

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
