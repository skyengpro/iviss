use crate::errors::AppError;
use crate::models::search_vehicle::VehicleRow;
use sqlx::{PgPool, Row};

pub async fn get_vehicle_with_owner_by_plate(
    pool: &PgPool,
    plate_number: &str,
) -> Result<Option<VehicleRow>, AppError> {
    let query = r#"
        SELECT 
            v.plate_number,
            v.chassis_number,
            v.brand,
            v.model,
            v.year,
            v.color,
            v.engine_power,
            v.fuel_type,
            vo.name as owner_name,
            vo.address as owner_address,
            vo.national_id as owner_national_id,
            NULL as carte_grise_expiry
        FROM vehicles v
        LEFT JOIN vehicle_owners vo ON v.id = vo.vehicle_id AND vo.is_current_owner = true
        WHERE v.plate_number = $1 AND v.deleted_at IS NULL AND (vo.deleted_at IS NULL OR vo.deleted_at IS NULL)
        LIMIT 1
    "#;

    let row = sqlx::query(query)
        .bind(plate_number)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;

    match row {
        Some(row) => {
            let vehicle_row = VehicleRow {
                plate_number: row.get("plate_number"),
                chassis_number: row.get("chassis_number"),
                brand: row.get("brand"),
                model: row.get("model"),
                year: row.get("year"),
                color: row.get("color"),
                engine_power: row.get("engine_power"),
                fuel_type: row.get("fuel_type"),
                owner_name: row.get("owner_name"),
                owner_address: row.get("owner_address"),
                owner_national_id: row.get("owner_national_id"),
                carte_grise_expiry: row.get("carte_grise_expiry"),
            };
            Ok(Some(vehicle_row))
        }
        None => Ok(None),
    }
}

pub async fn get_vehicle_status_by_plate(
    pool: &PgPool,
    plate_number: &str,
) -> Result<Option<VehicleStatusRow>, AppError> {
    let query = r#"
        SELECT 
            vs.insurance_status,
            vs.insurance_expiry,
            vs.technical_status,
            vs.technical_expiry,
            vs.stolen_status,
            vs.last_updated
        FROM vehicle_statuses vs
        JOIN vehicles v ON vs.vehicle_id = v.id
        WHERE v.plate_number = $1
        LIMIT 1
    "#;

    let row = sqlx::query(query)
        .bind(plate_number)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?;

    match row {
        Some(row) => {
            let status_row = VehicleStatusRow {
                insurance_status: row.get("insurance_status"),
                insurance_expiry: row.get("insurance_expiry"),
                technical_status: row.get("technical_status"),
                technical_expiry: row.get("technical_expiry"),
                stolen_status: row.get("stolen_status"),
                last_updated: row.get("last_updated"),
            };
            Ok(Some(status_row))
        }
        None => Ok(None),
    }
}

#[derive(Debug)]
pub struct VehicleStatusRow {
    pub insurance_status: Option<String>,
    pub insurance_expiry: Option<time::Date>,
    pub technical_status: Option<String>,
    pub technical_expiry: Option<time::Date>,
    pub stolen_status: bool,
    pub last_updated: Option<time::OffsetDateTime>,
}
