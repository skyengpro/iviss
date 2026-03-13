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
            vs.vehicle_image_url,
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
                vehicle_image_url: row.get("vehicle_image_url"),
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
    pub vehicle_image_url: Option<String>,
    #[allow(dead_code)]
    pub last_updated: Option<time::OffsetDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::search_vehicle::VehicleRow;
    use time::{Date, OffsetDateTime};

    // Helper function to create test vehicle row
    fn create_test_vehicle_row() -> VehicleRow {
        VehicleRow {
            plate_number: "TEST123".to_string(),
            chassis_number: "1HGBH41JXMN109186".to_string(),
            brand: "Toyota".to_string(),
            model: "Camry".to_string(),
            year: 2020,
            color: Some("Blue".to_string()),
            engine_power: Some("150 HP".to_string()),
            fuel_type: Some("Gasoline".to_string()),
            owner_name: "John Doe".to_string(),
            owner_address: Some("123 Main St".to_string()),
            owner_national_id: Some("1234567890".to_string()),
            carte_grise_expiry: Some("2024-12-31".to_string()),
        }
    }

    // Helper function to create test status row
    fn create_test_status_row() -> VehicleStatusRow {
        VehicleStatusRow {
            insurance_status: Some("valid".to_string()),
            insurance_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            technical_status: Some("valid".to_string()),
            technical_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            stolen_status: false,
            vehicle_image_url: Some("http://example.com/vehicle.jpg".to_string()),
            last_updated: Some(OffsetDateTime::now_utc()),
        }
    }

    #[test]
    fn test_vehicle_status_row_creation() {
        let status_row = create_test_status_row();

        // Test all fields are set correctly
        assert_eq!(status_row.insurance_status, Some("valid".to_string()));
        assert!(status_row.insurance_expiry.is_some());
        assert_eq!(status_row.technical_status, Some("valid".to_string()));
        assert!(status_row.technical_expiry.is_some());
        assert!(!status_row.stolen_status);
        assert_eq!(
            status_row.vehicle_image_url,
            Some("http://example.com/vehicle.jpg".to_string())
        );
        assert!(status_row.last_updated.is_some());
    }

    #[test]
    fn test_vehicle_status_row_with_optional_fields_none() {
        let status_row = VehicleStatusRow {
            insurance_status: None,
            insurance_expiry: None,
            technical_status: None,
            technical_expiry: None,
            stolen_status: false,
            vehicle_image_url: None,
            last_updated: None,
        };

        assert!(status_row.insurance_status.is_none());
        assert!(status_row.insurance_expiry.is_none());
        assert!(status_row.technical_status.is_none());
        assert!(status_row.technical_expiry.is_none());
        assert!(!status_row.stolen_status);
        assert!(status_row.vehicle_image_url.is_none());
        assert!(status_row.last_updated.is_none());
    }

    #[test]
    fn test_vehicle_status_row_stolen_vehicle() {
        let status_row = VehicleStatusRow {
            insurance_status: Some("valid".to_string()),
            insurance_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            technical_status: Some("valid".to_string()),
            technical_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            stolen_status: true,
            vehicle_image_url: Some("http://example.com/vehicle.jpg".to_string()),
            last_updated: Some(OffsetDateTime::now_utc()),
        };

        assert!(status_row.stolen_status);
        assert_eq!(status_row.insurance_status, Some("valid".to_string()));
        assert_eq!(status_row.technical_status, Some("valid".to_string()));
    }

    #[test]
    fn test_vehicle_status_row_with_expired_insurance() {
        let status_row = VehicleStatusRow {
            insurance_status: Some("expired".to_string()),
            insurance_expiry: Some(Date::from_ordinal_date(2023, 365).unwrap()), // Past date
            technical_status: Some("valid".to_string()),
            technical_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            stolen_status: false,
            vehicle_image_url: None,
            last_updated: Some(OffsetDateTime::now_utc()),
        };

        assert_eq!(status_row.insurance_status, Some("expired".to_string()));
        assert!(status_row.insurance_expiry.is_some());
        assert_eq!(status_row.technical_status, Some("valid".to_string()));
        assert!(!status_row.stolen_status);
    }

    #[test]
    fn test_vehicle_status_row_with_failed_technical() {
        let status_row = VehicleStatusRow {
            insurance_status: Some("valid".to_string()),
            insurance_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            technical_status: Some("failed".to_string()),
            technical_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()),
            stolen_status: false,
            vehicle_image_url: None,
            last_updated: Some(OffsetDateTime::now_utc()),
        };

        assert_eq!(status_row.insurance_status, Some("valid".to_string()));
        assert_eq!(status_row.technical_status, Some("failed".to_string()));
        assert!(!status_row.stolen_status);
    }

    #[test]
    fn test_vehicle_row_structure() {
        let vehicle_row = create_test_vehicle_row();

        // Test all required fields
        assert_eq!(vehicle_row.plate_number, "TEST123");
        assert_eq!(vehicle_row.chassis_number, "1HGBH41JXMN109186");
        assert_eq!(vehicle_row.brand, "Toyota");
        assert_eq!(vehicle_row.model, "Camry");
        assert_eq!(vehicle_row.year, 2020);
        assert_eq!(vehicle_row.owner_name, "John Doe");

        // Test optional fields
        assert_eq!(vehicle_row.color, Some("Blue".to_string()));
        assert_eq!(vehicle_row.engine_power, Some("150 HP".to_string()));
        assert_eq!(vehicle_row.fuel_type, Some("Gasoline".to_string()));
        assert_eq!(vehicle_row.owner_address, Some("123 Main St".to_string()));
        assert_eq!(
            vehicle_row.owner_national_id,
            Some("1234567890".to_string())
        );
        assert_eq!(
            vehicle_row.carte_grise_expiry,
            Some("2024-12-31".to_string())
        );
    }

    #[test]
    fn test_vehicle_row_with_optional_fields_none() {
        let vehicle_row = VehicleRow {
            plate_number: "TEST456".to_string(),
            chassis_number: "2HGBH41JXMN109187".to_string(),
            brand: "Honda".to_string(),
            model: "Civic".to_string(),
            year: 2021,
            color: None,
            engine_power: None,
            fuel_type: None,
            owner_name: "Jane Smith".to_string(),
            owner_address: None,
            owner_national_id: None,
            carte_grise_expiry: None,
        };

        assert_eq!(vehicle_row.plate_number, "TEST456");
        assert_eq!(vehicle_row.brand, "Honda");
        assert_eq!(vehicle_row.model, "Civic");
        assert_eq!(vehicle_row.year, 2021);
        assert_eq!(vehicle_row.owner_name, "Jane Smith");

        // Test optional fields are None
        assert!(vehicle_row.color.is_none());
        assert!(vehicle_row.engine_power.is_none());
        assert!(vehicle_row.fuel_type.is_none());
        assert!(vehicle_row.owner_address.is_none());
        assert!(vehicle_row.owner_national_id.is_none());
        assert!(vehicle_row.carte_grise_expiry.is_none());
    }

    #[test]
    fn test_vehicle_status_debug_format() {
        let status_row = create_test_status_row();
        let debug_str = format!("{:?}", status_row);

        // Debug format should contain key field information
        assert!(debug_str.contains("VehicleStatusRow"));
        assert!(debug_str.contains("valid"));
        assert!(debug_str.contains("false")); // stolen_status
    }

    #[test]
    fn test_vehicle_row_debug_format() {
        let vehicle_row = create_test_vehicle_row();
        let debug_str = format!("{:?}", vehicle_row);

        // Debug format should contain key field information
        assert!(debug_str.contains("VehicleRow"));
        assert!(debug_str.contains("TEST123"));
        assert!(debug_str.contains("Toyota"));
        assert!(debug_str.contains("Camry"));
        assert!(debug_str.contains("2020"));
    }

    #[test]
    fn test_date_handling_in_status_row() {
        let test_date = Date::from_ordinal_date(2024, 182).unwrap(); // Mid-year date
        let status_row = VehicleStatusRow {
            insurance_status: None,
            insurance_expiry: Some(test_date),
            technical_status: None,
            technical_expiry: Some(test_date),
            stolen_status: false,
            vehicle_image_url: None,
            last_updated: None,
        };

        assert_eq!(status_row.insurance_expiry, Some(test_date));
        assert_eq!(status_row.technical_expiry, Some(test_date));
    }

    #[test]
    fn test_offset_date_time_handling() {
        let test_datetime = OffsetDateTime::now_utc();
        let status_row = VehicleStatusRow {
            insurance_status: None,
            insurance_expiry: None,
            technical_status: None,
            technical_expiry: None,
            stolen_status: false,
            vehicle_image_url: None,
            last_updated: Some(test_datetime),
        };

        assert_eq!(status_row.last_updated, Some(test_datetime));
    }

    #[test]
    fn test_string_status_variations() {
        let test_cases = vec![
            ("valid", "valid"),
            ("expired", "expired"),
            ("failed", "failed"),
            ("none", "none"),
            ("pending", "pending"),
            ("unknown", "unknown"),
        ];

        for (input, expected) in test_cases {
            let status_row = VehicleStatusRow {
                insurance_status: Some(input.to_string()),
                insurance_expiry: None,
                technical_status: Some(input.to_string()),
                technical_expiry: None,
                stolen_status: false,
                vehicle_image_url: None,
                last_updated: None,
            };

            assert_eq!(status_row.insurance_status, Some(expected.to_string()));
            assert_eq!(status_row.technical_status, Some(expected.to_string()));
        }
    }

    #[test]
    fn test_vehicle_image_url_handling() {
        let test_urls = vec![
            Some("http://example.com/image1.jpg".to_string()),
            Some("https://secure.example.com/image2.png".to_string()),
            Some("data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQ".to_string()),
            None,
        ];

        for url in test_urls {
            let status_row = VehicleStatusRow {
                insurance_status: None,
                insurance_expiry: None,
                technical_status: None,
                technical_expiry: None,
                stolen_status: false,
                vehicle_image_url: url.clone(),
                last_updated: None,
            };

            assert_eq!(status_row.vehicle_image_url, url);
        }
    }
}
