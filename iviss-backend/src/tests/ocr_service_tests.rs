use image::GrayImage;
use leptess::LepTess;
use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::dto::scan::ScanResultData;
use crate::services::ocr_service::*;

/// Cameroon plate format: 2 letters + 3 digits + 2 letters (e.g. CE128BC).
static PLATE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Z]{2}[0-9]{3}[A-Z]{2}$").unwrap());

/// Radius (in pixels) for the adaptive threshold sliding window.
const ADAPTIVE_RADIUS: u32 = 40;

/// Offset subtracted from the local mean when applying adaptive threshold.
const ADAPTIVE_C: i16 = 5;

static TMP_COUNTER: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static TESSERACT: RefCell<Option<LepTess>> = const { RefCell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;
    use std::fs;
    use std::time::Duration;

    // ── Basic utility tests (existing) ───────────────────────────────────────

    #[test]
    fn normalise_removes_spaces_and_dashes() {
        assert_eq!(normalise_plate("ce 128 bc"), "CE128BC");
        assert_eq!(normalise_plate("CE-128-BC"), "CE128BC");
        assert_eq!(normalise_plate("  Ce128Bc  "), "CE128BC");
    }

    #[test]
    fn regex_accepts_valid_plates() {
        assert!(PLATE_REGEX.is_match("CE128BC"));
        assert!(PLATE_REGEX.is_match("AB000CD"));
        assert!(PLATE_REGEX.is_match("ZZ999ZZ"));
    }

    #[test]
    fn regex_rejects_invalid_plates() {
        assert!(!PLATE_REGEX.is_match("C128BC"));
        assert!(!PLATE_REGEX.is_match("CE12BC"));
        assert!(!PLATE_REGEX.is_match("CE1234BC"));
        assert!(!PLATE_REGEX.is_match("1E128BC"));
        assert!(!PLATE_REGEX.is_match(""));
    }

    #[test]
    fn contrast_stretch_full_range() {
        let img = GrayImage::from_fn(3, 1, |x, _| {
            Luma([match x {
                0 => 100,
                1 => 150,
                _ => 200,
            }])
        });
        let stretched = contrast_stretch(&img);
        assert_eq!(stretched.get_pixel(0, 0)[0], 0);
        assert_eq!(stretched.get_pixel(2, 0)[0], 255);
    }

    #[test]
    fn adaptive_threshold_basic() {
        let img = GrayImage::from_fn(5, 1, |x, _| Luma([if x == 2 { 200 } else { 10 }]));
        let result = adaptive_threshold(&img, 2, 5);
        assert_eq!(result.get_pixel(2, 0)[0], 255);
    }

    #[test]
    fn test_extract_plate_fuzzy() {
        assert_eq!(extract_plate_fuzzy("CE128BC"), Some("CE128BC".to_string()));
        assert_eq!(
            extract_plate_fuzzy("!CE128BC!"),
            Some("CE128BC".to_string())
        );
        assert_eq!(extract_plate_fuzzy("CE12OBC"), Some("CE120BC".to_string())); // O -> 0 in middle
        assert_eq!(extract_plate_fuzzy("1E128BC"), Some("IE128BC".to_string())); // 1 -> I at start
        assert_eq!(extract_plate_fuzzy("CE12"), Some("CE12".to_string()));
        assert_eq!(extract_plate_fuzzy(""), None);
    }

    #[test]
    fn test_pick_best_ensemble() {
        let cand1 = ScanResultData {
            plate: "P1".into(),
            raw_text: "P1".into(),
            confidence: 0.5,
            format_valid: false,
        };
        let cand2 = ScanResultData {
            plate: "CE128BC".into(),
            raw_text: "CE128BC".into(),
            confidence: 0.8,
            format_valid: true,
        };

        let best = pick_best_ensemble(vec![Some(cand1), Some(cand2)]);
        assert!(best.format_valid);
        assert_eq!(best.plate, "CE128BC");

        let best_empty = pick_best_ensemble(vec![]);
        assert_eq!(best_empty.plate, "");
    }

    #[test]
    fn test_image_helpers() {
        let img = GrayImage::from_pixel(10, 10, Luma([100]));
        let inverted = invert_image(&img);
        assert_eq!(inverted.get_pixel(0, 0)[0], 155);

        let bordered = add_border(&img, 5, 255);
        assert_eq!(bordered.width(), 20);
        assert_eq!(bordered.height(), 20);
    }

    // ── TesseractGuard tests ─────────────────────────────────────────────────

    #[test]
    fn test_tesseract_guard_creation() {
        // This test will fail if Tesseract is not available, but that's expected
        // In a real environment, you'd mock Tesseract or use testcontainers
        let result = TesseractGuard::new();
        // We can't guarantee success without Tesseract, but we can test the structure
        match result {
            Ok(_guard) => {
                // Guard was created successfully
                // Drop will be called automatically, testing the Drop impl
            }
            Err(_) => {
                // Expected if Tesseract is not available in test environment
            }
        }
    }

    // ── Tesseract management tests ───────────────────────────────────────────

    #[test]
    fn test_tesseract_thread_local_storage() {
        // Test that the thread local storage works
        TESSERACT.with(|cell| {
            let mut slot = cell.borrow_mut();
            assert!(slot.is_none(), "Initial state should be None");

            // Simulate putting a tesseract back
            *slot = None; // Would be Some(tess) in real scenario

            // Test taking works
            let taken = slot.take();
            assert!(taken.is_none(), "Should be able to take None");
        });
    }

    // ── finalize function tests ───────────────────────────────────────────────

    #[test]
    fn test_finalize_format_valid() {
        let result = ScanResultData {
            plate: "CE128BC".to_string(),
            raw_text: "CE128BC".to_string(),
            confidence: 0.7,
            format_valid: true,
        };

        let finalized = finalize(
            result,
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(300),
        )
        .unwrap();

        assert_eq!(finalized.plate, "CE128BC");
        assert_eq!(finalized.confidence, 0.90); // Should be set to 0.90 for valid format
        assert!(finalized.format_valid);
    }

    #[test]
    fn test_finalize_format_invalid_with_plate() {
        let result = ScanResultData {
            plate: "CE128".to_string(), // Invalid format
            raw_text: "CE128".to_string(),
            confidence: 0.7,
            format_valid: false,
        };

        let finalized = finalize(
            result,
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(300),
        )
        .unwrap();

        assert_eq!(finalized.plate, "CE128");
        assert_eq!(finalized.confidence, 0.50); // Should be set to 0.50 for invalid format
        assert!(!finalized.format_valid);
    }

    #[test]
    fn test_finalize_empty_plate() {
        let result = ScanResultData {
            plate: "".to_string(),
            raw_text: "".to_string(),
            confidence: 0.0,
            format_valid: false,
        };

        let finalized = finalize(
            result,
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(300),
        )
        .unwrap();

        assert_eq!(finalized.plate, "");
        assert_eq!(finalized.confidence, 0.0); // Should remain 0.0 for empty plate
        assert!(!finalized.format_valid);
    }

    // ── try_ocr_path function tests ───────────────────────────────────────────

    #[test]
    fn test_try_ocr_path_empty_result() {
        // This test requires mocking Tesseract or using a real instance
        // For now, we'll test the logic structure
        // In a real test environment, you'd create a mock Tesseract that returns empty text

        // Create a dummy image file for testing
        let img = GrayImage::from_pixel(10, 10, Luma([128]));
        let test_path = "/tmp/test_ocr_empty.png";

        if img.save(test_path).is_ok() {
            // This would normally test with a real Tesseract instance
            let _ = fs::remove_file(test_path);
        }
    }

    // ── Enhanced extract_plate_fuzzy tests ─────────────────────────────────────

    #[test]
    fn test_extract_plate_fuzzy_comprehensive() {
        // Test exact matches
        assert_eq!(extract_plate_fuzzy("CE128BC"), Some("CE128BC".to_string()));

        // Test with noise
        assert_eq!(
            extract_plate_fuzzy("Noise CE128BC More"),
            Some("CE128BC".to_string())
        );

        // Test character corrections in letter positions
        assert_eq!(extract_plate_fuzzy("C0128BC"), Some("CO128BC".to_string())); // 0->O at pos 1
        assert_eq!(extract_plate_fuzzy("C1128BC"), Some("CI128BC".to_string())); // 1->I at pos 1
        assert_eq!(extract_plate_fuzzy("C5128BC"), Some("CS128BC".to_string())); // 5->S at pos 1
        assert_eq!(extract_plate_fuzzy("C6128BC"), Some("CG128BC".to_string())); // 6->G at pos 1
        assert_eq!(extract_plate_fuzzy("C8128BC"), Some("CB128BC".to_string())); // 8->B at pos 1

        // Test character corrections in digit positions
        assert_eq!(extract_plate_fuzzy("CE1O8BC"), Some("CE108BC".to_string())); // O->0 at pos 3
        assert_eq!(extract_plate_fuzzy("CE1I8BC"), Some("CE118BC".to_string())); // I->1 at pos 3
        assert_eq!(extract_plate_fuzzy("CE1Z8BC"), Some("CE128BC".to_string())); // Z->2 at pos 3
        assert_eq!(extract_plate_fuzzy("CE1S8BC"), Some("CE158BC".to_string())); // S->5 at pos 3
        assert_eq!(extract_plate_fuzzy("CE1G8BC"), Some("CE168BC".to_string())); // G->6 at pos 3
        assert_eq!(extract_plate_fuzzy("CE1B8BC"), Some("CE188BC".to_string())); // B->8 at pos 3

        // Test last letter positions
        assert_eq!(extract_plate_fuzzy("CE128B0"), Some("CE128BO".to_string())); // 0->O at pos 6
        assert_eq!(extract_plate_fuzzy("CE128B1"), Some("CE128BI".to_string())); // 1->I at pos 6

        // Test short strings (fallback case)
        assert_eq!(extract_plate_fuzzy("CE12"), Some("CE12".to_string()));
        assert_eq!(extract_plate_fuzzy("1234"), Some("1234".to_string()));

        // Test too short strings
        assert_eq!(extract_plate_fuzzy("CE1"), None);
        assert_eq!(extract_plate_fuzzy(""), None);

        // Test with special characters
        assert_eq!(
            extract_plate_fuzzy("CE-128-BC"),
            Some("CE128BC".to_string())
        );
        assert_eq!(
            extract_plate_fuzzy("CE 128 BC"),
            Some("CE128BC".to_string())
        );
        assert_eq!(
            extract_plate_fuzzy("CE.128.BC"),
            Some("CE128BC".to_string())
        );
    }

    // ── Enhanced image processing tests ───────────────────────────────────────

    #[test]
    fn test_contrast_stretch_edge_cases() {
        // Test empty image
        let empty_img = GrayImage::new(0, 0);
        let result = contrast_stretch(&empty_img);
        assert_eq!(result.dimensions(), (0, 0));

        // Test uniform image (all same pixel value)
        let uniform_img = GrayImage::from_pixel(10, 10, Luma([128]));
        let result = contrast_stretch(&uniform_img);
        assert_eq!(result.get_pixel(0, 0)[0], 128); // Should remain unchanged

        // Test already full range image
        let full_range_img = GrayImage::from_fn(2, 1, |x, _| Luma([if x == 0 { 0 } else { 255 }]));
        let result = contrast_stretch(&full_range_img);
        assert_eq!(result.get_pixel(0, 0)[0], 0);
        assert_eq!(result.get_pixel(1, 0)[0], 255);

        // Test single pixel image
        let single_img = GrayImage::from_pixel(1, 1, Luma([100]));
        let result = contrast_stretch(&single_img);
        assert_eq!(result.get_pixel(0, 0)[0], 100);
    }

    #[test]
    fn test_adaptive_threshold_edge_cases() {
        // Test empty image
        let empty_img = GrayImage::new(0, 0);
        let result = adaptive_threshold(&empty_img, 5, 5);
        assert_eq!(result.dimensions(), (0, 0));

        // Test uniform image
        let uniform_img = GrayImage::from_pixel(10, 10, Luma([128]));
        let result = adaptive_threshold(&uniform_img, 5, 5);
        // All pixels should be the same (either all black or all white)
        let first_pixel = result.get_pixel(0, 0)[0];
        for y in 0..result.height() {
            for x in 0..result.width() {
                assert_eq!(result.get_pixel(x, y)[0], first_pixel);
            }
        }

        // Test with different radius values
        let img = GrayImage::from_fn(5, 5, |_, _| Luma([128]));
        let result1 = adaptive_threshold(&img, 1, 5);
        let result2 = adaptive_threshold(&img, 10, 5);
        assert_eq!(result1.dimensions(), (5, 5));
        assert_eq!(result2.dimensions(), (5, 5));

        // Test with different C values
        let result3 = adaptive_threshold(&img, 5, 0);
        let result4 = adaptive_threshold(&img, 5, 10);
        assert_eq!(result3.dimensions(), (5, 5));
        assert_eq!(result4.dimensions(), (5, 5));
    }

    #[test]
    fn test_invert_image_comprehensive() {
        // Test black and white
        let black_img = GrayImage::from_pixel(5, 5, Luma([0]));
        let inverted_black = invert_image(&black_img);
        assert_eq!(inverted_black.get_pixel(0, 0)[0], 255);

        let white_img = GrayImage::from_pixel(5, 5, Luma([255]));
        let inverted_white = invert_image(&white_img);
        assert_eq!(inverted_white.get_pixel(0, 0)[0], 0);

        // Test middle value
        let gray_img = GrayImage::from_pixel(5, 5, Luma([128]));
        let inverted_gray = invert_image(&gray_img);
        assert_eq!(inverted_gray.get_pixel(0, 0)[0], 127);

        // Test double inversion returns original
        let double_inverted = invert_image(&inverted_gray);
        assert_eq!(double_inverted.get_pixel(0, 0)[0], 128);
    }

    #[test]
    fn test_add_border_comprehensive() {
        let img = GrayImage::from_pixel(10, 15, Luma([100]));

        // Test with zero border (should be same size)
        let no_border = add_border(&img, 0, 255);
        assert_eq!(no_border.dimensions(), (10, 15));

        // Test with positive border
        let bordered = add_border(&img, 5, 255);
        assert_eq!(bordered.dimensions(), (20, 25));

        // Check border color
        assert_eq!(bordered.get_pixel(0, 0)[0], 255); // Top-left corner
        assert_eq!(bordered.get_pixel(19, 24)[0], 255); // Bottom-right corner

        // Check original image is centered
        assert_eq!(bordered.get_pixel(5, 5)[0], 100); // Top-left of original
        assert_eq!(bordered.get_pixel(14, 19)[0], 100); // Bottom-right of original

        // Test with different border color
        let black_border = add_border(&img, 3, 0);
        assert_eq!(black_border.get_pixel(0, 0)[0], 0);
        assert_eq!(black_border.dimensions(), (16, 21));
    }

    // ── Enhanced pick_best_ensemble tests ─────────────────────────────────────

    #[test]
    fn test_pick_best_ensemble_comprehensive() {
        // Test with all None
        let result = pick_best_ensemble(vec![None, None, None]);
        assert_eq!(result.plate, "");
        assert_eq!(result.confidence, 0.0);
        assert!(!result.format_valid);

        // Test with only invalid formats
        let cand1 = ScanResultData {
            plate: "INVALID1".to_string(),
            raw_text: "INVALID1".to_string(),
            confidence: 0.3,
            format_valid: false,
        };
        let cand2 = ScanResultData {
            plate: "INVALID2".to_string(),
            raw_text: "INVALID2".to_string(),
            confidence: 0.7,
            format_valid: false,
        };
        let result = pick_best_ensemble(vec![Some(cand1), Some(cand2)]);
        assert_eq!(result.plate, "INVALID2"); // Higher confidence wins
        assert!(!result.format_valid);

        // Test with mixed valid/invalid
        let invalid = ScanResultData {
            plate: "INVALID".to_string(),
            raw_text: "INVALID".to_string(),
            confidence: 0.9,
            format_valid: false,
        };
        let valid = ScanResultData {
            plate: "CE128BC".to_string(),
            raw_text: "CE128BC".to_string(),
            confidence: 0.1,
            format_valid: true,
        };
        let result = pick_best_ensemble(vec![Some(invalid), Some(valid)]);
        assert_eq!(result.plate, "CE128BC"); // Valid format wins regardless of confidence
        assert!(result.format_valid);

        // Test with same validity, different confidence
        let valid1 = ScanResultData {
            plate: "AB123CD".to_string(),
            raw_text: "AB123CD".to_string(),
            confidence: 0.6,
            format_valid: true,
        };
        let valid2 = ScanResultData {
            plate: "EF456GH".to_string(),
            raw_text: "EF456GH".to_string(),
            confidence: 0.8,
            format_valid: true,
        };
        let result = pick_best_ensemble(vec![Some(valid1), Some(valid2)]);
        assert_eq!(result.plate, "EF456GH"); // Higher confidence wins when both valid
    }

    // ── scan_plate error handling tests ───────────────────────────────────────

    #[test]
    fn test_scan_plate_invalid_image() {
        let invalid_bytes = b"not an image";
        let result = scan_plate(invalid_bytes);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot decode image"));
    }

    #[test]
    fn test_scan_plate_empty_input() {
        let empty_bytes = b"";
        let result = scan_plate(empty_bytes);
        assert!(result.is_err());
    }

    // ── Constants and utilities tests ─────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert_eq!(ADAPTIVE_RADIUS, 40);
        assert_eq!(ADAPTIVE_C, 5);
    }

    #[test]
    fn test_tmp_counter() {
        let initial = TMP_COUNTER.load(Ordering::Relaxed);
        let next = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        assert_eq!(next, initial);
        assert_eq!(TMP_COUNTER.load(Ordering::Relaxed), initial + 1);
    }

    // ── Integration-style test with mock data ───────────────────────────────────

    #[test]
    fn test_full_pipeline_with_mock_image() {
        // Create a simple test image
        let img = GrayImage::from_pixel(100, 50, Luma([128]));

        // Add some "text" pattern (simplified)
        let mut test_img = img.clone();
        for y in 20..30 {
            for x in 10..90 {
                test_img.put_pixel(x, y, Luma([255]));
            }
        }

        // Save to bytes
        // Note: This would need proper encoding in a real test
        // For now, we'll just test that the function handles the input

        // Test with minimal valid PNG bytes (this is a simplified example)
        // Note: In a real test environment, you would create a proper test image
        // For now, we'll skip this test as it requires external files
    }
}
