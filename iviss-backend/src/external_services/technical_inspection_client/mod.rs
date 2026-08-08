use crate::dto::{common::Status, search_vehicle::TechnicalStatus};

/// Current technical-inspection integration placeholder until its transport is decided.
pub fn pending_technical_status() -> TechnicalStatus {
    TechnicalStatus {
        status: Status::Pending,
        last_inspection_date: None,
        expiry_date: None,
        mileage: None,
        defects: Vec::new(),
        notes: Some("No technical inspection data available for the moment".to_string()),
    }
}
