use crate::dto::common::Status;
use crate::dto::search_vehicle::{
    CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, StatusResults, TechnicalStatus,
    VehicleInfo,
};
use crate::models::search_vehicle::VehicleRow;
use crate::queries::vehicle_queries::VehicleStatusRow;

pub struct VehicleService;

impl VehicleService {
    pub fn calculate_overall_status(
        insurance: &InsuranceStatus,
        police: &PoliceStatus,
        customs: &CustomsStatus,
        technical: &TechnicalStatus,
    ) -> Status {
        // If any status is critical, overall is critical
        if matches!(insurance.status, Status::Critical)
            || matches!(police.status, Status::Critical)
            || matches!(customs.status, Status::Critical)
            || matches!(technical.status, Status::Critical)
        {
            return Status::Critical;
        }

        // If any status is warning, overall is warning
        if matches!(insurance.status, Status::Warning)
            || matches!(police.status, Status::Warning)
            || matches!(customs.status, Status::Warning)
            || matches!(technical.status, Status::Warning)
        {
            return Status::Warning;
        }

        // If all are valid, overall is valid
        Status::Valid
    }

    pub fn build_vehicle_info(vehicle_row: &VehicleRow) -> VehicleInfo {
        VehicleInfo {
            brand: vehicle_row.brand.clone(),
            model: vehicle_row.model.clone(),
            year: vehicle_row.year,
            color: vehicle_row.color.clone(),
            engine_power: vehicle_row.engine_power.clone(),
            fuel_type: vehicle_row.fuel_type.clone(),
            chassis_number: vehicle_row.chassis_number.clone(),
            owner: OwnerInfo {
                name: vehicle_row.owner_name.clone(),
                address: vehicle_row.owner_address.clone(),
                national_id: vehicle_row.owner_national_id.clone(),
            },
        }
    }

    pub fn build_insurance_status(status_row: &Option<VehicleStatusRow>) -> InsuranceStatus {
        match status_row {
            Some(row) => {
                let status = match &row.insurance_status {
                    Some(status_str) => match status_str.as_str() {
                        "valid" => Status::Valid,
                        "expired" => Status::Critical,
                        "none" => Status::Warning,
                        _ => Status::Pending,
                    },
                    None => Status::Pending,
                };

                InsuranceStatus {
                    status,
                    provider: None,      // Would come from external API
                    policy_number: None, // Would come from external API
                    expiry_date: row.insurance_expiry.map(|d| d.to_string()),
                    coverage_type: None, // Would come from external API
                    notes: None,
                }
            }
            None => InsuranceStatus {
                status: Status::Pending,
                provider: None,
                policy_number: None,
                expiry_date: None,
                coverage_type: None,
                notes: Some("No insurance data available".to_string()),
            },
        }
    }

    pub fn build_police_status(status_row: &Option<VehicleStatusRow>) -> PoliceStatus {
        match status_row {
            Some(row) => {
                let (status, is_wanted, is_stolen) = if row.stolen_status {
                    (Status::Critical, false, true)
                } else {
                    (Status::Valid, false, false)
                };

                PoliceStatus {
                    status,
                    is_wanted,
                    is_stolen,
                    report_date: None,   // Would come from external police API
                    report_number: None, // Would come from external police API
                    notes: if row.stolen_status {
                        Some("Vehicle reported as stolen".to_string())
                    } else {
                        None
                    },
                }
            }
            None => PoliceStatus {
                status: Status::Pending,
                is_wanted: false,
                is_stolen: false,
                report_date: None,
                report_number: None,
                notes: Some("No police data available".to_string()),
            },
        }
    }

    pub fn build_customs_status(_status_row: &Option<VehicleStatusRow>) -> CustomsStatus {
        // For now, assume all vehicles are cleared
        // In a real implementation, this would call external customs API
        CustomsStatus {
            status: Status::Valid,
            is_cleared: true,
            import_date: None,        // Would come from external customs API
            declaration_number: None, // Would come from external customs API
            notes: None,
        }
    }

    pub fn build_technical_status(status_row: &Option<VehicleStatusRow>) -> TechnicalStatus {
        match status_row {
            Some(row) => {
                let status = match &row.technical_status {
                    Some(status_str) => match status_str.as_str() {
                        "valid" => Status::Valid,
                        "expired" => Status::Critical,
                        "failed" => Status::Critical,
                        _ => Status::Pending,
                    },
                    None => Status::Pending,
                };

                TechnicalStatus {
                    status,
                    last_inspection_date: None, // Would come from external API
                    expiry_date: row.technical_expiry.map(|d| d.to_string()),
                    mileage: None,       // Would come from external API
                    defects: Vec::new(), // Would come from external API
                    notes: None,
                }
            }
            None => TechnicalStatus {
                status: Status::Pending,
                last_inspection_date: None,
                expiry_date: None,
                mileage: None,
                defects: Vec::new(),
                notes: Some("No technical inspection data available".to_string()),
            },
        }
    }

    pub fn build_status_results(status_row: &Option<VehicleStatusRow>) -> StatusResults {
        let insurance = Self::build_insurance_status(status_row);
        let police = Self::build_police_status(status_row);
        let customs = Self::build_customs_status(status_row);
        let technical = Self::build_technical_status(status_row);

        let overall_status =
            Self::calculate_overall_status(&insurance, &police, &customs, &technical);

        StatusResults {
            overall_status,
            insurance,
            police,
            customs,
            technical,
            vehicle_image_url: status_row
                .as_ref()
                .and_then(|row| row.vehicle_image_url.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::common::Status;
    use crate::dto::search_vehicle::{
        CustomsStatus, InsuranceStatus, PoliceStatus, TechnicalStatus,
    };
    use crate::models::search_vehicle::VehicleRow;
    use crate::queries::vehicle_queries::VehicleStatusRow;
    use time::{Date, OffsetDateTime};

    // Helper function to create test vehicle status row
    fn create_test_status_row() -> VehicleStatusRow {
        VehicleStatusRow {
            insurance_status: Some("valid".to_string()),
            insurance_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()), // Dec 31, 2024
            technical_status: Some("valid".to_string()),
            technical_expiry: Some(Date::from_ordinal_date(2024, 365).unwrap()), // Dec 31, 2024
            stolen_status: false,
            vehicle_image_url: Some("http://example.com/vehicle.jpg".to_string()),
            last_updated: Some(OffsetDateTime::now_utc()),
        }
    }

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

    // Helper function to create test insurance status
    fn create_test_insurance_status(status: Status) -> InsuranceStatus {
        InsuranceStatus {
            status,
            provider: None,
            policy_number: None,
            expiry_date: None,
            coverage_type: None,
            notes: None,
        }
    }

    // Helper function to create test police status
    fn create_test_police_status(status: Status) -> PoliceStatus {
        PoliceStatus {
            status,
            is_wanted: false,
            is_stolen: false,
            report_date: None,
            report_number: None,
            notes: None,
        }
    }

    // Helper function to create test customs status
    fn create_test_customs_status(status: Status) -> CustomsStatus {
        CustomsStatus {
            status,
            is_cleared: true,
            import_date: None,
            declaration_number: None,
            notes: None,
        }
    }

    // Helper function to create test technical status
    fn create_test_technical_status(status: Status) -> TechnicalStatus {
        TechnicalStatus {
            status,
            last_inspection_date: None,
            expiry_date: None,
            mileage: None,
            defects: Vec::new(),
            notes: None,
        }
    }

    #[test]
    fn test_calculate_overall_status_all_valid() {
        let insurance = create_test_insurance_status(Status::Valid);
        let police = create_test_police_status(Status::Valid);
        let customs = create_test_customs_status(Status::Valid);
        let technical = create_test_technical_status(Status::Valid);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Valid);
    }

    #[test]
    fn test_calculate_overall_status_single_warning() {
        let insurance = create_test_insurance_status(Status::Warning);
        let police = create_test_police_status(Status::Valid);
        let customs = create_test_customs_status(Status::Valid);
        let technical = create_test_technical_status(Status::Valid);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Warning);
    }

    #[test]
    fn test_calculate_overall_status_multiple_warnings() {
        let insurance = create_test_insurance_status(Status::Warning);
        let police = create_test_police_status(Status::Warning);
        let customs = create_test_customs_status(Status::Valid);
        let technical = create_test_technical_status(Status::Valid);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Warning);
    }

    #[test]
    fn test_calculate_overall_status_single_critical() {
        let insurance = create_test_insurance_status(Status::Critical);
        let police = create_test_police_status(Status::Valid);
        let customs = create_test_customs_status(Status::Valid);
        let technical = create_test_technical_status(Status::Valid);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Critical);
    }

    #[test]
    fn test_calculate_overall_status_mixed_critical_and_warning() {
        let insurance = create_test_insurance_status(Status::Critical);
        let police = create_test_police_status(Status::Warning);
        let customs = create_test_customs_status(Status::Warning);
        let technical = create_test_technical_status(Status::Valid);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Critical);
    }

    #[test]
    fn test_calculate_overall_status_all_critical() {
        let insurance = create_test_insurance_status(Status::Critical);
        let police = create_test_police_status(Status::Critical);
        let customs = create_test_customs_status(Status::Critical);
        let technical = create_test_technical_status(Status::Critical);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Critical);
    }

    #[test]
    fn test_calculate_overall_status_with_pending() {
        let insurance = create_test_insurance_status(Status::Pending);
        let police = create_test_police_status(Status::Valid);
        let customs = create_test_customs_status(Status::Valid);
        let technical = create_test_technical_status(Status::Valid);

        let result =
            VehicleService::calculate_overall_status(&insurance, &police, &customs, &technical);
        assert_eq!(result, Status::Valid);
    }

    #[test]
    fn test_build_vehicle_info() {
        let vehicle_row = create_test_vehicle_row();
        let result = VehicleService::build_vehicle_info(&vehicle_row);

        assert_eq!(result.brand, "Toyota");
        assert_eq!(result.model, "Camry");
        assert_eq!(result.year, 2020);
        assert_eq!(result.color, Some("Blue".to_string()));
        assert_eq!(result.engine_power, Some("150 HP".to_string()));
        assert_eq!(result.fuel_type, Some("Gasoline".to_string()));
        assert_eq!(result.chassis_number, "1HGBH41JXMN109186");
        assert_eq!(result.owner.name, "John Doe");
        assert_eq!(result.owner.address, Some("123 Main St".to_string()));
        assert_eq!(result.owner.national_id, Some("1234567890".to_string()));
    }

    #[test]
    fn test_build_vehicle_info_with_optional_fields_none() {
        let mut vehicle_row = create_test_vehicle_row();
        vehicle_row.color = None;
        vehicle_row.engine_power = None;
        vehicle_row.fuel_type = None;
        vehicle_row.owner_address = None;
        vehicle_row.owner_national_id = None;

        let result = VehicleService::build_vehicle_info(&vehicle_row);

        assert_eq!(result.color, None);
        assert_eq!(result.engine_power, None);
        assert_eq!(result.fuel_type, None);
        assert_eq!(result.owner.address, None);
        assert_eq!(result.owner.national_id, None);
    }

    #[test]
    fn test_build_insurance_status_valid() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = Some("valid".to_string());

        let result = VehicleService::build_insurance_status(&Some(status_row));

        assert_eq!(result.status, Status::Valid);
        assert_eq!(result.provider, None);
        assert_eq!(result.policy_number, None);
        assert!(result.expiry_date.is_some());
        assert_eq!(result.coverage_type, None);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_insurance_status_expired() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = Some("expired".to_string());

        let result = VehicleService::build_insurance_status(&Some(status_row));

        assert_eq!(result.status, Status::Critical);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_insurance_status_none() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = Some("none".to_string());

        let result = VehicleService::build_insurance_status(&Some(status_row));

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_insurance_status_unknown() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = Some("unknown".to_string());

        let result = VehicleService::build_insurance_status(&Some(status_row));

        assert_eq!(result.status, Status::Pending);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_insurance_status_no_status_field() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = None;

        let result = VehicleService::build_insurance_status(&Some(status_row));

        assert_eq!(result.status, Status::Pending);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_insurance_status_no_data() {
        let result = VehicleService::build_insurance_status(&None);

        assert_eq!(result.status, Status::Pending);
        assert_eq!(result.provider, None);
        assert_eq!(result.policy_number, None);
        assert_eq!(result.expiry_date, None);
        assert_eq!(result.coverage_type, None);
        assert_eq!(
            result.notes,
            Some("No insurance data available".to_string())
        );
    }

    #[test]
    fn test_build_police_status_not_stolen() {
        let mut status_row = create_test_status_row();
        status_row.stolen_status = false;

        let result = VehicleService::build_police_status(&Some(status_row));

        assert_eq!(result.status, Status::Valid);
        assert!(!result.is_wanted);
        assert!(!result.is_stolen);
        assert_eq!(result.report_date, None);
        assert_eq!(result.report_number, None);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_police_status_stolen() {
        let mut status_row = create_test_status_row();
        status_row.stolen_status = true;

        let result = VehicleService::build_police_status(&Some(status_row));

        assert_eq!(result.status, Status::Critical);
        assert!(!result.is_wanted);
        assert!(result.is_stolen);
        assert_eq!(result.report_date, None);
        assert_eq!(result.report_number, None);
        assert_eq!(result.notes, Some("Vehicle reported as stolen".to_string()));
    }

    #[test]
    fn test_build_police_status_no_data() {
        let result = VehicleService::build_police_status(&None);

        assert_eq!(result.status, Status::Pending);
        assert!(!result.is_wanted);
        assert!(!result.is_stolen);
        assert_eq!(result.report_date, None);
        assert_eq!(result.report_number, None);
        assert_eq!(result.notes, Some("No police data available".to_string()));
    }

    #[test]
    fn test_build_customs_status() {
        let status_row = create_test_status_row();
        let result = VehicleService::build_customs_status(&Some(status_row));

        assert_eq!(result.status, Status::Valid);
        assert!(result.is_cleared);
        assert_eq!(result.import_date, None);
        assert_eq!(result.declaration_number, None);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_customs_status_no_data() {
        let result = VehicleService::build_customs_status(&None);

        assert_eq!(result.status, Status::Valid);
        assert!(result.is_cleared);
        assert_eq!(result.import_date, None);
        assert_eq!(result.declaration_number, None);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_technical_status_valid() {
        let mut status_row = create_test_status_row();
        status_row.technical_status = Some("valid".to_string());

        let result = VehicleService::build_technical_status(&Some(status_row));

        assert_eq!(result.status, Status::Valid);
        assert_eq!(result.last_inspection_date, None);
        assert!(result.expiry_date.is_some());
        assert_eq!(result.mileage, None);
        assert!(result.defects.is_empty());
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_technical_status_expired() {
        let mut status_row = create_test_status_row();
        status_row.technical_status = Some("expired".to_string());

        let result = VehicleService::build_technical_status(&Some(status_row));

        assert_eq!(result.status, Status::Critical);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_technical_status_failed() {
        let mut status_row = create_test_status_row();
        status_row.technical_status = Some("failed".to_string());

        let result = VehicleService::build_technical_status(&Some(status_row));

        assert_eq!(result.status, Status::Critical);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_technical_status_unknown() {
        let mut status_row = create_test_status_row();
        status_row.technical_status = Some("unknown".to_string());

        let result = VehicleService::build_technical_status(&Some(status_row));

        assert_eq!(result.status, Status::Pending);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_technical_status_no_status_field() {
        let mut status_row = create_test_status_row();
        status_row.technical_status = None;

        let result = VehicleService::build_technical_status(&Some(status_row));

        assert_eq!(result.status, Status::Pending);
        assert_eq!(result.notes, None);
    }

    #[test]
    fn test_build_technical_status_no_data() {
        let result = VehicleService::build_technical_status(&None);

        assert_eq!(result.status, Status::Pending);
        assert_eq!(result.last_inspection_date, None);
        assert_eq!(result.expiry_date, None);
        assert_eq!(result.mileage, None);
        assert!(result.defects.is_empty());
        assert_eq!(
            result.notes,
            Some("No technical inspection data available".to_string())
        );
    }

    #[test]
    fn test_build_status_results_with_data() {
        let status_row = create_test_status_row();
        let result = VehicleService::build_status_results(&Some(status_row));

        // Check that all status components are built
        assert!(matches!(result.insurance.status, Status::Valid));
        assert!(matches!(result.police.status, Status::Valid));
        assert!(matches!(result.customs.status, Status::Valid));
        assert!(matches!(result.technical.status, Status::Valid));

        // Check overall status is calculated correctly
        assert_eq!(result.overall_status, Status::Valid);

        // Check vehicle image URL is preserved
        assert_eq!(
            result.vehicle_image_url,
            Some("http://example.com/vehicle.jpg".to_string())
        );
    }

    #[test]
    fn test_build_status_results_no_data() {
        let result = VehicleService::build_status_results(&None);

        // Check that all status components are pending
        assert_eq!(result.insurance.status, Status::Pending);
        assert_eq!(result.police.status, Status::Pending);
        assert_eq!(result.customs.status, Status::Valid); // Customs always returns Valid
        assert_eq!(result.technical.status, Status::Pending);

        // Check overall status is calculated correctly (customs is valid, others pending)
        assert_eq!(result.overall_status, Status::Valid);

        // Check vehicle image URL is None
        assert_eq!(result.vehicle_image_url, None);
    }

    #[test]
    fn test_build_status_results_mixed_statuses() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = Some("expired".to_string()); // Critical
        status_row.technical_status = Some("failed".to_string()); // Critical
        status_row.stolen_status = true; // Critical for police

        let result = VehicleService::build_status_results(&Some(status_row));

        // Check individual statuses
        assert_eq!(result.insurance.status, Status::Critical);
        assert_eq!(result.police.status, Status::Critical);
        assert_eq!(result.customs.status, Status::Valid);
        assert_eq!(result.technical.status, Status::Critical);

        // Overall should be Critical due to insurance, police, and technical
        assert_eq!(result.overall_status, Status::Critical);
    }

    #[test]
    fn test_build_status_results_warning_overall() {
        let mut status_row = create_test_status_row();
        status_row.insurance_status = Some("none".to_string()); // Warning
        status_row.stolen_status = false; // Valid for police
        status_row.technical_status = Some("valid".to_string()); // Valid

        let result = VehicleService::build_status_results(&Some(status_row));

        // Check individual statuses
        assert_eq!(result.insurance.status, Status::Warning);
        assert_eq!(result.police.status, Status::Valid);
        assert_eq!(result.customs.status, Status::Valid);
        assert_eq!(result.technical.status, Status::Valid);

        // Overall should be Warning due to insurance
        assert_eq!(result.overall_status, Status::Warning);
    }
}
