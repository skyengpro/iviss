use crate::dto::scan::ScanResultData;
use crate::services::photo_ocr_service::{enhance_photo_result, extract_plate_strict, pick_best};

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_plate_strict: unaffected by this round, kept as a guard ────

    #[test]
    fn extract_plate_strict_recovers_a_plate_from_noisy_text() {
        assert_eq!(
            extract_plate_strict("some text PA 02 RC 521 more text"),
            Some("PA02RC521".to_string())
        );
    }

    #[test]
    fn extract_plate_strict_returns_none_without_a_plate() {
        assert_eq!(extract_plate_strict("just some text"), None);
    }

    // ── enhance_photo_result: format_valid derived, confidence never touched ─

    #[test]
    fn enhance_photo_result_keeps_an_already_valid_plate_untouched() {
        let input = ScanResultData {
            plate: "CE128BC".to_string(),
            raw_text: "some text CE128BC".to_string(),
            confidence: 0.12, // realistic field regime, not the old 0.90 fixture value
            format_valid: true,
            plate_type: Some("civil_cemac".to_string()),
        };
        let output = enhance_photo_result(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn enhance_photo_result_derives_format_valid_instead_of_asserting_it() {
        // extract_plate_strict cannot guarantee the string it returns
        // classifies; `enhance_photo_result` must derive `format_valid` from
        // `classify`, not assert it unconditionally to `true`.
        let input = ScanResultData {
            plate: "".to_string(),
            raw_text: "CE128BC".to_string(),
            confidence: 0.30,
            format_valid: false,
            plate_type: None,
        };
        let output = enhance_photo_result(input);
        assert_eq!(output.plate, "CE128BC".to_string());
        assert!(output.format_valid);
        assert_eq!(output.plate_type, Some("civil_cemac".to_string()));
    }

    #[test]
    fn enhance_photo_result_never_synthesises_confidence() {
        // The old code floored confidence to 0.90 whenever a strict
        // extraction succeeded. `confidence` must stay exactly what the OCR
        // pass measured, on this path as on `ocr_service::finalize`.
        for confidence in [0.0f32, 0.12, 0.95] {
            let input = ScanResultData {
                plate: "".to_string(),
                raw_text: "CE128BC".to_string(),
                confidence,
                format_valid: false,
                plate_type: None,
            };
            let output = enhance_photo_result(input);
            assert_eq!(
                output.confidence, confidence,
                "confidence must not be rewritten (input was {confidence})"
            );
        }
    }

    #[test]
    fn enhance_photo_result_leaves_unparseable_plate_alone() {
        let input = ScanResultData {
            plate: "AB123C".to_string(), // non-empty but not a valid format
            raw_text: "some CE128BC text".to_string(),
            confidence: 0.16,
            format_valid: false,
            plate_type: None,
        };
        let output = enhance_photo_result(input);
        assert_eq!(output.plate, "AB123C".to_string());
        assert!(!output.format_valid);
        assert_eq!(output.confidence, 0.16);
    }

    // ── pick_best: no more character-level vote ─────────────────────────────

    #[test]
    fn pick_best_never_fabricates_a_composite_plate() {
        // Both invalid, same length, disagreeing on one character. The old
        // vote built a third string from the two and could promote it to
        // `format_valid = true` — a plate neither pass actually read.
        let a = ScanResultData {
            plate: "CE568LB".to_string(),
            raw_text: "CE568LB".to_string(),
            confidence: 0.40,
            format_valid: false,
            plate_type: None,
        };
        let b = ScanResultData {
            plate: "CE568LR".to_string(),
            raw_text: "CE568LR".to_string(),
            confidence: 0.20,
            format_valid: false,
            plate_type: None,
        };
        let result = pick_best(a.clone(), b.clone());
        // Selection must return one of the two readings exactly as recognised.
        assert!(result.plate == a.plate || result.plate == b.plate);
        assert_eq!(result.confidence, a.confidence.max(b.confidence));
    }

    #[test]
    fn pick_best_prefers_valid_format_over_confidence() {
        let a = ScanResultData {
            plate: "P1".to_string(),
            raw_text: "".to_string(),
            confidence: 0.5,
            format_valid: true,
            plate_type: None,
        };
        let b = ScanResultData {
            plate: "P2".to_string(),
            raw_text: "".to_string(),
            confidence: 0.9,
            format_valid: false,
            plate_type: None,
        };
        assert_eq!(pick_best(a.clone(), b.clone()).plate, "P1".to_string());
        assert_eq!(pick_best(b, a).plate, "P1".to_string());
    }

    #[test]
    fn pick_best_prefers_a_candidate_with_text_over_an_empty_one() {
        let empty = ScanResultData {
            plate: "".to_string(),
            raw_text: "".to_string(),
            confidence: 0.9,
            format_valid: false,
            plate_type: None,
        };
        let with_text = ScanResultData {
            plate: "P2".to_string(),
            raw_text: "".to_string(),
            confidence: 0.1,
            format_valid: false,
            plate_type: None,
        };
        assert_eq!(
            pick_best(empty.clone(), with_text.clone()).plate,
            "P2".to_string()
        );
        assert_eq!(pick_best(with_text, empty).plate, "P2".to_string());
    }
}
