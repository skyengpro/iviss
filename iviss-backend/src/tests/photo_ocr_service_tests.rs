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
}
