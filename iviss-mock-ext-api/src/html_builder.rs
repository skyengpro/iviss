//! HTML response builder for iviss-test-api.
//!
//! Reconstructs the exact HTML fragment that the real external vehicle
//! registry API returns inside the `{"data": "..."}` JSON envelope.
//! The format is derived directly from the production response sample and
//! confirmed against `parser.rs` in `iviss-backend`.

use crate::db::Vehicle;

/// Build the JSON-wrapped HTML response for a found vehicle, exactly
/// matching the real API's format.
///
/// Example output (pretty-printed for readability):
/// ```text
/// {"data": "<font color='red'>...(NOT_CLEARED!)...</font><br>---- * * * ----<br>..."}
/// ```
pub fn build_found_response(v: &Vehicle) -> String {
    let html = build_html(v);
    // Escape any quotes inside the HTML string for safe JSON embedding.
    // The real API does not escape — we replicate that by using serde_json
    // which handles it correctly.
    let json = serde_json::json!({ "data": html });
    json.to_string()
}

/// Build the JSON-wrapped "not found / service indisponible" response.
/// The exact format is taken from the real API's not-found payload and
/// is what `parser.rs::is_vehicle_not_found_response()` detects.
pub fn build_not_found_response(plate: &str) -> String {
    let timestamp = current_timestamp();
    let data = format!(
        "\n---------\nService distant:\n---------\n{plate}\n--> Service indisponible\n{timestamp}"
    );
    serde_json::json!({ "data": data }).to_string()
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_html(v: &Vehicle) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Clearance banner (first line)
    let is_not_cleared = v
        .customs_status
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("NOT_CLEARED"))
        .unwrap_or(false);

    if is_not_cleared {
        parts.push(
            "<font color='red'><b>&nbsp;&nbsp;(NOT_CLEARED!)</b></font>".to_string(),
        );
    } else {
        parts.push(
            "<font color='green'><b>&nbsp;&nbsp;(CLEARED)</b></font>".to_string(),
        );
    }

    parts.push("---- * * * ----".to_string());

    // Immat (plate number — always present)
    parts.push(labeled_field("Immat", &v.plate_number));

    // Optional fields — only emitted when present in the DB row
    if let Some(chassis) = &v.chassis_number {
        parts.push(labeled_field("Chassis", chassis));
    }
    if let Some(mt) = &v.mark_and_type {
        // Label key uses HTML entity for the ampersand, matching real API
        parts.push(labeled_field("M&amp;T", mt));
    }
    if let Some(power) = &v.engine_power {
        parts.push(labeled_field("Puissance", power));
    }
    if let Some(owner) = &v.owner_name {
        parts.push(labeled_field("Prop", owner));
    }
    if let Some(nps) = &v.nps_status {
        parts.push(labeled_field("Statut NPS", nps));
    }

    // Customs status (coloured)
    let douane_color = if is_not_cleared { "red" } else { "black" };
    let customs_val = v.customs_status.as_deref().unwrap_or("CLEARED");
    parts.push(format!(
        "<font color='black'><b>Statut DOUANE:&nbsp;&nbsp;</b></font><font color='{douane_color}'><b>{customs_val}</b></font>"
    ));

    parts.join("<br>")
}

fn labeled_field(key: &str, value: &str) -> String {
    format!(
        "<font color='black'><b>{key}:&nbsp;&nbsp;</b></font><font color='gray'><b>{value}</b></font>"
    )
}

fn current_timestamp() -> String {
    use time::OffsetDateTime;
    let fmt = time::macros::format_description!("[day]-[month]-[year] [hour]:[minute]:[second]");
    OffsetDateTime::now_utc()
        .format(&fmt)
        .unwrap_or_else(|_| "00-00-0000 00:00:00".to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_vehicle(customs_status: &str) -> Vehicle {
        Vehicle {
            plate_number:   "CE 568 LR".to_string(),
            chassis_number: Some("WDB4632341X258849".to_string()),
            mark_and_type:  Some("MERCEDES AB54E2".to_string()),
            engine_power:   Some("19".to_string()),
            owner_name:     Some("VESSAH MOHAMED".to_string()),
            nps_status:     Some("RAS".to_string()),
            customs_status: Some(customs_status.to_string()),
        }
    }

    #[test]
    fn not_cleared_banner_present() {
        let html = build_found_response(&sample_vehicle("NOT_CLEARED"));
        assert!(html.contains("NOT_CLEARED!"));
        assert!(html.contains("color='red'"));
    }

    #[test]
    fn cleared_banner_present() {
        let html = build_found_response(&sample_vehicle("CLEARED"));
        assert!(html.contains("(CLEARED)"));
        assert!(html.contains("color='green'"));
    }

    #[test]
    fn immat_always_present() {
        let html = build_found_response(&sample_vehicle("CLEARED"));
        assert!(html.contains("CE 568 LR"));
        assert!(html.contains("Immat"));
    }

    #[test]
    fn mark_and_type_uses_html_entity() {
        let html = build_found_response(&sample_vehicle("CLEARED"));
        // The label key should use the HTML entity, not a raw &
        assert!(html.contains("M&amp;T"));
    }

    #[test]
    fn optional_fields_omitted_when_none() {
        let v = Vehicle {
            plate_number:   "CE 999 ED".to_string(),
            chassis_number: None,
            mark_and_type:  None,
            engine_power:   None,
            owner_name:     None,
            nps_status:     None,
            customs_status: Some("CLEARED".to_string()),
        };
        let html = build_found_response(&v);
        assert!(!html.contains("Chassis"));
        assert!(!html.contains("M&amp;T"));
        assert!(!html.contains("Puissance"));
        assert!(!html.contains("Prop"));
        assert!(!html.contains("Statut NPS"));
    }

    #[test]
    fn not_found_contains_service_indisponible() {
        let resp = build_not_found_response("XX 999 ZZ");
        assert!(resp.contains("Service indisponible"));
        assert!(resp.contains("XX 999 ZZ"));
    }
}
