use reqwest;
use tracing::{debug, info};

pub struct VehicleApiCredentials {
    pub base_url: String,
    pub user_auth: ApiUserAuth,
    pub header_parms: ExternalApiHeaderParms,
}
#[derive(Debug, Clone)]
pub struct ExternalApiHeaderParms {
    pub user: String,
    pub lock_ndia: String,
    pub kindia: String,
    pub client: String,
    pub ctr: String,
}

#[derive(Debug, Clone)]
pub struct ApiUserAuth {
    pub username: String,
    pub password: String,
}
#[derive(Debug, Clone)]
pub struct VehicleApiServise {
    pub credentials: VehicleApiCredentials,
    pub client: reqwest::Client,
}
impl VehicleApiServise {
    pub fn new(api_credentials: VehicleApiCredentials) -> Self {
        Self {
            credentials: api_credentials,
            client: reqwest::Client::new(),
        }
    }

    pub query_plate(&self, plate: &str) -> anyhow::Result<String> {
        debug!("Querying vehicle API for plate: {}", plate);
        let url = format!("{}/query?plate={}", self.credentials.base_url, plate);
        let response = self
            .client
            .post(&url)
            .basic_auth(
                &self.credentials.user_auth.username,
                Some(&self.credentials.user_auth.password),
            )
            .header("X-User", &self.credentials.header_parms.user)
            .header("X-Lock-NDIA", &self.credentials.header_parms.lock_ndia)
            .header("X-Kindia", &self.credentials.header_parms.kindia)
            .header("X-Client", &self.credentials.header_parms.client)
            .header("X-CTR", &self.credentials.header_parms.ctr)
            .send()?
            .error_for_status()?;

        let vehicle_info = response.text()?;
        debug!("Received response from vehicle API for plate {}: {}", plate, vehicle_info);
        Ok(vehicle_info)
    }
    fn parse_html_response(&self, html: &str) -> anyhow::Result<String> {
        // Implement HTML parsing logic here to extract necessary information
        // For example, you might use the `scraper` crate to parse the HTML and extract data
        Ok(html.to_string()) // Placeholder: return the raw HTML for now
    }
}
