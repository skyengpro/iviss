use crate::dto::search_vehicle::{OwnerInfo, VehicleInfo};
use crate::vehicle_client::parser::{
    html_to_text, is_vehicle_not_found_response, parse_inline_customs_status,
    parse_label_value_lines, split_brand_and_model,
};
use crate::vehicle_client::types::{
    ExternalVehicle, VehicleApiCredentials, VehicleApiError, VehicleApiResponse,
};
use anyhow::{anyhow, Context};
use std::time::Duration;
use tracing::debug;

/// HTTP client wrapper for the external vehicle registry API.
#[derive(Debug, Clone)]
pub struct VehicleApiService {
    pub credentials: VehicleApiCredentials,
    pub client: reqwest::Client,
}

impl VehicleApiService {
    /// Build a new service, optionally pinning the TLS root certificate
    /// supplied as a base64-encoded PEM string.
    pub fn new(api_credentials: VehicleApiCredentials) -> anyhow::Result<Self> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let mut client_builder = reqwest::Client::builder()
            .http1_only()
            .timeout(Duration::from_secs(15));
        let tls_cert_b64 = api_credentials.tls_cert_b64.trim();

        if !tls_cert_b64.is_empty() {
            let cert_pem = STANDARD
                .decode(tls_cert_b64)
                .context("Failed to decode EXTERNAL_API_TLS_CERT_B64")?;

            let cert = reqwest::Certificate::from_pem(&cert_pem)
                .context("Failed to parse TLS certificate")?;
            client_builder = client_builder.add_root_certificate(cert);
        }

        Ok(Self {
            credentials: api_credentials,
            client: client_builder
                .build()
                .context("failed to build vehicle API HTTP client")?,
        })
    }

    /// Query the external registry for a single plate number.
    pub async fn query_plate(&self, plate: &str) -> Result<VehicleApiResponse, VehicleApiError> {
        debug!("Querying vehicle API for plate");
        let url = format!("{}/query", self.credentials.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .basic_auth(
                &self.credentials.user_auth.username,
                Some(&self.credentials.user_auth.password),
            )
            .header("user", &self.credentials.header_parms.user)
            .header("lockNdia", &self.credentials.header_parms.lock_ndia)
            .header("kindia", &self.credentials.header_parms.kindia)
            .header("client", &self.credentials.header_parms.client)
            .header("ctr", &self.credentials.header_parms.ctr)
            .header(reqwest::header::CONTENT_TYPE, "text/plain")
            .body(plate.to_owned())
            .send()
            .await
            .context("failed to call vehicle API")?
            .error_for_status()
            .context("vehicle API returned an error status")?;

        let html = response
            .text()
            .await
            .context("failed to read vehicle API response")?;
        debug!("Received vehicle API response ({} bytes)", html.len());

        if is_vehicle_not_found_response(&html) {
            return Err(VehicleApiError::NotFound);
        }

        self.parse_html_response(&html).map_err(Into::into)
    }

    /// Fetch all vehicles matching a prefix from the external registry.
    pub async fn fetch_batch(&self, prefix: &str) -> anyhow::Result<Vec<ExternalVehicle>> {
        debug!("Fetching batch from vehicle API for prefix: {}", prefix);
        let url = format!("{}/batch", self.credentials.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&url)
            .query(&[("prefix", prefix)])
            .basic_auth(
                &self.credentials.user_auth.username,
                Some(&self.credentials.user_auth.password),
            )
            .header("user", &self.credentials.header_parms.user)
            .header("lockNdia", &self.credentials.header_parms.lock_ndia)
            .header("kindia", &self.credentials.header_parms.kindia)
            .header("client", &self.credentials.header_parms.client)
            .header("ctr", &self.credentials.header_parms.ctr)
            .send()
            .await
            .context("failed to call vehicle API batch endpoint")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "vehicle API batch endpoint returned HTTP {} for prefix '{}': {}",
                status,
                prefix,
                body.trim()
            ));
        }

        let vehicles = response
            .json::<Vec<ExternalVehicle>>()
            .await
            .context("failed to deserialize batch vehicle list")?;

        Ok(vehicles)
    }

    pub fn parse_html_response(&self, html: &str) -> anyhow::Result<VehicleApiResponse> {
        let text = html_to_text(html);
        let fields = parse_label_value_lines(&text);

        let mt = fields.get("M&T").cloned();
        let (brand, model) = split_brand_and_model(mt.as_deref());
        let customs_status = fields
            .get("STATUT DOUANE")
            .cloned()
            .or_else(|| parse_inline_customs_status(&text));

        let vehicle = VehicleInfo {
            brand,
            model,
            year: None,
            color: None,
            engine_power: fields.get("PUISSANCE").cloned(),
            fuel_type: None,
            chassis_number: fields.get("CHASSIS").cloned(),
            customs_status,
            owner: OwnerInfo {
                name: fields.get("PROP").cloned(),
                address: None,
                national_id: None,
            },
        };

        if !fields.contains_key("IMMAT")
            && vehicle.chassis_number.is_none()
            && vehicle.brand.is_none()
            && vehicle.owner.name.is_none()
        {
            return Err(anyhow!(
                "vehicle API response did not contain recognizable vehicle fields"
            ));
        }

        Ok(VehicleApiResponse {
            plate_number: fields.get("IMMAT").cloned(),
            vehicle,
        })
    }
}
