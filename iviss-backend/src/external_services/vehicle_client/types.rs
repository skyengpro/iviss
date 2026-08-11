use crate::dto::search_vehicle::VehicleInfo;

/// Credentials for the external vehicle registry API.
#[derive(Debug, Clone)]
pub struct VehicleApiCredentials {
    pub base_url: String,
    pub user_auth: ApiUserAuth,
    pub header_parms: ExternalApiHeaderParms,
    pub tls_cert_b64: String,
}

/// HTTP Basic-auth credentials for the external API.
#[derive(Debug, Clone)]
pub struct ApiUserAuth {
    pub username: String,
    pub password: String,
}

/// Custom HTTP headers required by the external vehicle registry API.
#[derive(Debug, Clone)]
pub struct ExternalApiHeaderParms {
    pub user: String,
    pub lock_ndia: String,
    pub kindia: String,
    pub client: String,
    pub ctr: String,
}

/// Errors that can occur when querying the external vehicle API.
#[derive(Debug, thiserror::Error)]
pub enum VehicleApiError {
    #[error("Vehicle not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Parsed response from the external vehicle API.
#[derive(Debug)]
pub struct VehicleApiResponse {
    pub plate_number: Option<String>,
    pub vehicle: VehicleInfo,
}
