use crate::dto::common::Status;
use crate::dto::search_vehicle::{
    CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, StatusResults, TechnicalStatus,
    VehicleInfo,
};
use crate::external_services::{
    insurance_client::pending_insurance_status,
    technical_inspection_client::pending_technical_status,
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

        // If any status is pending, overall is pending
        if matches!(insurance.status, Status::Pending)
            || matches!(police.status, Status::Pending)
            || matches!(customs.status, Status::Pending)
            || matches!(technical.status, Status::Pending)
        {
            return Status::Pending;
        }

        Status::Valid
    }

    /// Build status results from a live external API response (no DB row).
    ///
    /// Insurance, police, and technical statuses are `Pending` until real
    /// external data sources are integrated. Only the customs status is derived
    /// from the vehicle data returned by the registry API.
    pub fn build_status_results_from_api(vehicle_info: &VehicleInfo) -> StatusResults {
        let insurance = pending_insurance_status();
        let police = PoliceStatus {
            status: Status::Pending,
            is_wanted: false,
            is_stolen: false,
            report_date: None,
            report_number: None,
            notes: Some("No police data available for the moment".to_string()),
        };
        let customs = Self::build_customs_status_from_api(vehicle_info.customs_status.as_deref());
        let technical = pending_technical_status();
        let overall_status =
            Self::calculate_overall_status(&insurance, &police, &customs, &technical);

        StatusResults {
            overall_status,
            insurance,
            police,
            customs,
            technical,
            vehicle_image_url: None,
        }
    }

    /// Derive a [`CustomsStatus`] from the raw string returned by the
    /// external vehicle registry API.
    pub fn build_customs_status_from_api(customs_status: Option<&str>) -> CustomsStatus {
        match customs_status.map(|status| status.trim().to_uppercase()) {
            Some(status) if status == "CLEARED" || status == "OK" || status == "RAS" => {
                CustomsStatus {
                    status: Status::Valid,
                    is_cleared: true,
                    import_date: None,
                    declaration_number: None,
                    notes: Some(status),
                }
            }
            Some(status) if status == "NOT_CLEARED" => CustomsStatus {
                status: Status::Critical,
                is_cleared: false,
                import_date: None,
                declaration_number: None,
                notes: Some(status),
            },
            Some(status) => CustomsStatus {
                status: Status::Warning,
                is_cleared: false,
                import_date: None,
                declaration_number: None,
                notes: Some(status),
            },
            None => CustomsStatus {
                status: Status::Pending,
                is_cleared: false,
                import_date: None,
                declaration_number: None,
                notes: Some("No customs data available from the vehicle API".to_string()),
            },
        }
    }

    pub fn build_vehicle_info(vehicle_row: &VehicleRow) -> VehicleInfo {
        VehicleInfo {
            brand: Some(vehicle_row.brand.clone()),
            model: Some(vehicle_row.model.clone()),
            year: Some(vehicle_row.year),
            color: vehicle_row.color.clone(),
            engine_power: vehicle_row.engine_power.clone(),
            fuel_type: vehicle_row.fuel_type.clone(),
            chassis_number: Some(vehicle_row.chassis_number.clone()),
            customs_status: None,
            owner: OwnerInfo {
                name: Some(vehicle_row.owner_name.clone()),
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
