use crate::dto::search_vehicle::VehicleInfo;
use async_trait::async_trait;

pub mod insurance_client;
pub mod technical_inspection_client;
pub mod vehicle_client;

/// Payload returned by a partner data source.
#[derive(Debug)]
pub enum PartnerPayload {
    Vehicle {
        plate_number: Option<String>,
        vehicle: VehicleInfo,
    },
}

/// Result of a partner health probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
}

/// Errors exposed by partner data sources.
#[derive(Debug, thiserror::Error)]
pub enum ExternalServiceError {
    #[error("external record not found")]
    NotFound,
    #[error("external service unavailable: {0}")]
    Unavailable(String),
    #[error("external service protocol error: {0}")]
    Protocol(String),
}

/// Port implemented by real external data providers.
#[async_trait]
pub trait ExternalDataSource: Send + Sync {
    fn service_id(&self) -> &'static str;

    async fn fetch(&self, plate: &str) -> Result<PartnerPayload, ExternalServiceError>;

    async fn health_probe(&self) -> HealthStatus;
}
