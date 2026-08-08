use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Returns `true` when the API body signals that the plate is not registered.
pub fn is_vehicle_not_found_response(body: &str) -> bool {
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(data_str) = json_val.get("data").and_then(|v| v.as_str()) {
            return data_str.contains("Service indisponible");
        }
    }
    false
}

/// Strip HTML tags, converting `<br>` variants to newlines.
pub fn html_to_text(html: &str) -> String {
    static BR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<br\s*/?>").unwrap());
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<[^>]+>").unwrap());

    let with_breaks = BR_RE.replace_all(html, "\n");
    let without_tags = TAG_RE.replace_all(&with_breaks, "");

    decode_basic_html_entities(&without_tags)
}

/// Parse `LABEL: value` lines into a case-normalised map.
pub fn parse_label_value_lines(text: &str) -> HashMap<String, String> {
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

/// Split a `"BRAND MODEL"` string into `(brand, model)`.
pub fn split_brand_and_model(value: Option<&str>) -> (Option<String>, Option<String>) {
    match value.and_then(|v| v.split_once(char::is_whitespace)) {
        Some((brand, model)) => (
            Some(brand.trim().to_string()),
            Some(model.trim().to_string()),
        ),
        None => (value.map(|v| v.to_string()), None),
    }
}

/// Extract an inline customs status marker such as `(NOT_CLEARED!)` from the
/// full response text when it is not surfaced as a labelled field.
pub fn parse_inline_customs_status(text: &str) -> Option<String> {
    static INLINE_STATUS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(([A-Z_]+)!?\)").unwrap());

    INLINE_STATUS_RE
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|match_value| clean_value(match_value.as_str()))
        .filter(|value| !value.is_empty())
}

// ── private helpers ───────────────────────────────────────────────────────────

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

// ── unit tests (moved from vehicle_client_service.rs) ────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_services::vehicle_client::client::VehicleApiService;
    use crate::external_services::vehicle_client::types::{
        ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials,
    };

    #[test]
    fn test_is_vehicle_not_found_response_with_not_found_json() {
        let json_body = r#"{"data": "\n---------\nService distant:\n---------\nSN 49 02\n--> Service indisponible\n15-06-2026 09:33:09"}"#;
        assert!(is_vehicle_not_found_response(json_body));
    }

    #[test]
    fn test_is_vehicle_not_found_response_with_other_json() {
        let json_body = r#"{"data": "Some other error message"}"#;
        assert!(!is_vehicle_not_found_response(json_body));

        let json_body_no_data = r#"{"status": "error"}"#;
        assert!(!is_vehicle_not_found_response(json_body_no_data));
    }

    #[test]
    fn test_is_vehicle_not_found_response_with_html() {
        let html_body = "<html><body>IMMAT: SN 49 02</body></html>";
        assert!(!is_vehicle_not_found_response(html_body));
    }

    fn make_service() -> VehicleApiService {
        VehicleApiService {
            credentials: VehicleApiCredentials {
                base_url: "http://localhost".to_string(),
                user_auth: ApiUserAuth {
                    username: "user".to_string(),
                    password: "pass".to_string(),
                },
                header_parms: ExternalApiHeaderParms {
                    user: "u".to_string(),
                    lock_ndia: "l".to_string(),
                    kindia: "k".to_string(),
                    client: "c".to_string(),
                    ctr: "ctr".to_string(),
                },
                tls_cert_b64: "".to_string(),
            },
            client: reqwest::Client::new(),
        }
    }

    #[test]
    fn test_parse_html_response_success() {
        let html_body = "<html><body>IMMAT: SN 49 02<br/>M&T: TOYOTA COROLLA<br/>CHASSIS: JT123456789<br/>PROP: JOHN DOE</body></html>";
        let api_service = make_service();

        let result = api_service.parse_html_response(html_body).unwrap();
        assert_eq!(result.plate_number.as_deref(), Some("SN 49 02"));
        assert_eq!(result.vehicle.brand.as_deref(), Some("TOYOTA"));
        assert_eq!(result.vehicle.model.as_deref(), Some("COROLLA"));
        assert_eq!(
            result.vehicle.chassis_number.as_deref(),
            Some("JT123456789")
        );
        assert_eq!(result.vehicle.owner.name.as_deref(), Some("JOHN DOE"));
    }

    #[test]
    fn test_parse_html_response_missing_fields() {
        let html_body = "<html><body>No fields here</body></html>";
        let api_service = make_service();

        let result = api_service.parse_html_response(html_body);
        assert!(result.is_err());
    }
}
