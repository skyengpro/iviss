use crate::dto::{
    common::Status,
    search_vehicle::{
        CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, StatusResults, TechnicalStatus,
        VehicleInfo,
    },
};
use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct VehicleApiCredentials {
    pub base_url: String,
    pub user_auth: ApiUserAuth,
    pub header_parms: ExternalApiHeaderParms,
    pub tls_cert_b64: String,
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
pub struct VehicleApiService {
    pub credentials: VehicleApiCredentials,
    pub client: reqwest::Client,
}

#[derive(Debug)]
pub struct VehicleApiResponse {
    pub plate_number: Option<String>,
    pub vehicle: VehicleInfo,
}

impl VehicleApiService {
    pub fn new(api_credentials: VehicleApiCredentials) -> anyhow::Result<Self> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let cert_pem = STANDARD
            .decode(&api_credentials.tls_cert_b64)
            .context("Failed to decode EXTERNAL_API_TLS_CERT_B64")?;

        let cert =
            reqwest::Certificate::from_pem(&cert_pem).context("Failed to parse TLS certificate")?;
        Ok(Self {
            credentials: api_credentials,
            client: reqwest::Client::builder()
                .http1_only()
                .add_root_certificate(cert)
                .timeout(Duration::from_secs(25))
                .build()
                .context("failed to build vehicle API HTTP client")?,
        })
    }

    pub async fn query_plate(&self, plate: &str) -> anyhow::Result<VehicleApiResponse> {
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

        self.parse_html_response(&html)
    }

    fn parse_html_response(&self, html: &str) -> anyhow::Result<VehicleApiResponse> {
        let text = html_to_text(html);
        let fields = parse_label_value_lines(&text);

        let mt = fields.get("M&T").cloned();
        let (brand, model) = split_make_and_model(mt.as_deref());
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
    pub fn build_status_results_from_api(vehicle_info: &VehicleInfo) -> StatusResults {
        let insurance = InsuranceStatus {
            status: Status::Pending,
            provider: None,
            policy_number: None,
            expiry_date: None,
            coverage_type: None,
            notes: Some("No insurance data available for the moment".to_string()),
        };
        let police = PoliceStatus {
            status: Status::Pending,
            is_wanted: false,
            is_stolen: false,
            report_date: None,
            report_number: None,
            notes: Some("No police data available for the moment".to_string()),
        };
        let customs = Self::build_customs_status_from_api(vehicle_info.customs_status.as_deref());
        let technical = TechnicalStatus {
            status: Status::Pending,
            last_inspection_date: None,
            expiry_date: None,
            mileage: None,
            defects: Vec::new(),
            notes: Some("No technical inspection data available for the moment".to_string()),
        };
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

    fn calculate_overall_status(
        insurance: &InsuranceStatus,
        police: &PoliceStatus,
        customs: &CustomsStatus,
        technical: &TechnicalStatus,
    ) -> Status {
        if matches!(insurance.status, Status::Critical)
            || matches!(police.status, Status::Critical)
            || matches!(customs.status, Status::Critical)
            || matches!(technical.status, Status::Critical)
        {
            return Status::Critical;
        }

        if matches!(insurance.status, Status::Warning)
            || matches!(police.status, Status::Warning)
            || matches!(customs.status, Status::Warning)
            || matches!(technical.status, Status::Warning)
        {
            return Status::Warning;
        }

        if matches!(insurance.status, Status::Pending)
            || matches!(police.status, Status::Pending)
            || matches!(customs.status, Status::Pending)
            || matches!(technical.status, Status::Pending)
        {
            return Status::Pending;
        }

        Status::Valid
    }

    fn build_customs_status_from_api(customs_status: Option<&str>) -> CustomsStatus {
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
}

fn html_to_text(html: &str) -> String {
    static BR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<br\s*/?>").unwrap());
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<[^>]+>").unwrap());

    let with_breaks = BR_RE.replace_all(html, "\n");
    let without_tags = TAG_RE.replace_all(&with_breaks, "");

    decode_basic_html_entities(&without_tags)
}

fn parse_label_value_lines(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .filter_map(|(label, value)| {
            let label = label.trim().to_uppercase();
            let value = clean_value(value);

            if label.is_empty() || value.is_empty() {
                None
            } else {
                Some((label, value))
            }
        })
        .collect()
}

fn split_make_and_model(value: Option<&str>) -> (Option<String>, Option<String>) {
    match value.and_then(|v| v.split_once(char::is_whitespace)) {
        Some((brand, model)) => (
            Some(brand.trim().to_string()),
            Some(model.trim().to_string()),
        ),
        None => (value.map(|v| v.to_string()), None),
    }
}

fn parse_inline_customs_status(text: &str) -> Option<String> {
    static INLINE_STATUS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(([A-Z_]+)!?\)").unwrap());

    INLINE_STATUS_RE
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|match_value| clean_value(match_value.as_str()))
        .filter(|value| !value.is_empty())
}

fn clean_value(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '!'))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_basic_html_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
