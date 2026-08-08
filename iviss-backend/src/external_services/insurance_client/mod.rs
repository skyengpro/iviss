use crate::dto::{common::Status, search_vehicle::InsuranceStatus};

/// Current insurance integration placeholder until its transport is decided.
pub fn pending_insurance_status() -> InsuranceStatus {
    InsuranceStatus {
        status: Status::Pending,
        provider: None,
        policy_number: None,
        expiry_date: None,
        coverage_type: None,
        notes: Some("No insurance data available for the moment".to_string()),
    }
}
