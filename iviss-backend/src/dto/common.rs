use serde::{Deserialize, Serialize};

/// Shared status across all partner API responses
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Valid,
    Warning,
    Critical,
    Pending,
}

/// How the plate was identified by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum IdentificationMode {
    Manual,
    Photo,
    Live,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct SubmissionLocation {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
}
