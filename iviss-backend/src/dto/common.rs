use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Shared status across all partner API responses
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Valid,
    Warning,
    Critical,
    Pending,
}

/// How the plate was identified by the agent
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum IdentificationMode {
    Manual,
    Photo,
    Live,
}
