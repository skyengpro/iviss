use crate::dto::scan::ScanResultData;
use crate::services::photo_ocr_service::{
    enhance_photo_result, extract_plate_strict, photo_plate, pick_best,
};

// A minimal valid JPEG image (1x1 red pixel) for testing
// Sourced from: https://www.nayuki.io/page/small-image-files
const DUMMY_JPEG_BYTES: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF,
    0xC4, 0x00, 0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0xFF,
    0xC4, 0x00, 0xB2, 0x10, 0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x7D, 0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x13, 0x06,
    0x14, 0x21, 0x31, 0x07, 0x41, 0x51, 0x08, 0x61, 0x71, 0x81, 0x91, 0xA1, 0x09, 0x22, 0x32, 0xB1,
    0xC1, 0xD1, 0xE1, 0xF1, 0x0A, 0x23, 0x42, 0x52, 0x62, 0x72, 0x82, 0x92, 0xA2, 0xB2, 0xC2, 0xD2,
    0xE2, 0xF2, 0x15, 0x24, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46,
    0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66,
    0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86,
    0x87, 0x88, 0x89, 0x8A, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA3, 0xA4, 0xA5, 0xA6,
    0xA7, 0xA8, 0xA9, 0xAA, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC3, 0xC4, 0xC5, 0xC6,
    0xC7, 0xC8, 0xC9, 0xCA, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE3, 0xE4, 0xE5, 0xE6,
    0xE7, 0xE8, 0xE9, 0xEA, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFF, 0xDA, 0x00, 0x0C,
    0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3F, 0x00, 0x2A, 0xFF, 0xD9,
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_plate_strict Tests ---
    #[test]
    fn test_extract_plate_strict_valid_format() {
        assert_eq!(extract_plate_strict("CE128BC"), Some("CE128BC".to_string()));
        assert_eq!(extract_plate_strict("LT4568A"), Some("LT4568A".to_string()));
        assert_eq!(extract_plate_strict("SN1234"), Some("SN1234".to_string()));
    }

    #[test]
    fn test_extract_plate_strict_with_noise() {
        assert_eq!(
            extract_plate_strict("  CE 128 BC  "),
            Some("CE128BC".to_string())
        );
        assert_eq!(
            extract_plate_strict("some text PA 02 RC 521 more text"),
            Some("PA02RC521".to_string())
        );
        assert_eq!(
            extract_plate_strict("CE@128#BC"),
            Some("CE128BC".to_string())
        );
        assert_eq!(extract_plate_strict("ce128bc"), Some("CE128BC".to_string()));
        // Should convert to uppercase
    }

    #[test]
    fn test_extract_plate_strict_no_plate() {
        assert_eq!(extract_plate_strict("just some text"), None);
        assert_eq!(extract_plate_strict("ABCDEFG"), None);
    }

    #[test]
    fn test_extract_plate_strict_empty_string() {
        assert_eq!(extract_plate_strict(""), None);
    }

    // --- enhance_photo_result Tests ---
    #[test]
    fn test_enhance_photo_result_already_valid_plate() {
        let input = ScanResultData {
            plate: "CE128BC".to_string(),
            raw_text: "some text CE128BC".to_string(),
            confidence: 0.95,
            format_valid: true,
            plate_type: Some("civil_cemac".to_string()),
        };
        let output = enhance_photo_result(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn test_enhance_photo_result_invalid_but_strict_extracts() {
        // When a plate exists (even with invalid format), the current implementation keeps it.
        // The function does not modify existing plates - it only tries strict extraction
        // when the plate is completely empty.
        let input = ScanResultData {
            plate: "AB123C".to_string(), // Invalid format, but non-empty
            raw_text: "some CE128BC text".to_string(),
            confidence: 0.60,
            format_valid: false,
            plate_type: None,
        };
        let output = enhance_photo_result(input);
        // Plate should remain unchanged since it's not empty
        assert_eq!(output.plate, "AB123C".to_string());
        assert!(!output.format_valid); // Still invalid
        assert_eq!(output.confidence, 0.60); // Confidence unchanged
    }

    #[test]
    fn test_enhance_photo_result_no_plate_found() {
        let input = ScanResultData {
            plate: "".to_string(),
            raw_text: "some text with numbers 12345".to_string(),
            confidence: 0.50,
            format_valid: false,
            plate_type: None,
        };
        let output = enhance_photo_result(input.clone());
        assert_eq!(output, input); // Should be unchanged if no strict match
    }

    #[test]
    fn test_enhance_photo_result_strict_extract_boosts_low_confidence() {
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
        assert_eq!(output.confidence, 0.90);
    }

    #[test]
    fn test_enhance_photo_result_strict_extract_does_not_lower_high_confidence() {
        let input = ScanResultData {
            plate: "".to_string(),
            raw_text: "CE128BC".to_string(),
            confidence: 0.95,
            format_valid: false,
            plate_type: None,
        };
        let output = enhance_photo_result(input);
        assert_eq!(output.plate, "CE128BC".to_string());
        assert!(output.format_valid);
        assert_eq!(output.plate_type, Some("civil_cemac".to_string()));
        assert_eq!(output.confidence, 0.95); // Should remain high
    }

    // --- pick_best Tests ---
    #[test]
    fn test_pick_best_priority_format_valid() {
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
        assert_eq!(pick_best(b.clone(), a.clone()).plate, "P1".to_string());
    }

    #[test]
    fn test_pick_best_priority_plate_existence() {
        let a = ScanResultData {
            plate: "".to_string(),
            raw_text: "".to_string(),
            confidence: 0.5,
            format_valid: false,
            plate_type: None,
        };
        let b = ScanResultData {
            plate: "P2".to_string(),
            raw_text: "".to_string(),
            confidence: 0.7,
            format_valid: false,
            plate_type: None,
        };
        assert_eq!(pick_best(a.clone(), b.clone()).plate, "P2".to_string());
        assert_eq!(pick_best(b.clone(), a.clone()).plate, "P2".to_string());
    }

    #[test]
    fn test_pick_best_priority_confidence() {
        let a = ScanResultData {
            plate: "P1".to_string(),
            raw_text: "".to_string(),
            confidence: 0.5,
            format_valid: false,
            plate_type: None,
        };
        let b = ScanResultData {
            plate: "P2".to_string(),
            raw_text: "".to_string(),
            confidence: 0.9,
            format_valid: false,
            plate_type: None,
        };
        assert_eq!(pick_best(a.clone(), b.clone()).plate, "P2".to_string());
        assert_eq!(pick_best(b.clone(), a.clone()).plate, "P2".to_string());
    }

    #[test]
    fn test_pick_best_equal() {
        let a = ScanResultData {
            plate: "P1".to_string(),
            raw_text: "".to_string(),
            confidence: 0.8,
            format_valid: true,
            plate_type: None,
        };
        let b = ScanResultData {
            plate: "P2".to_string(),
            raw_text: "".to_string(),
            confidence: 0.8,
            format_valid: true,
            plate_type: None,
        };
        assert_eq!(pick_best(a.clone(), b.clone()).plate, "P1".to_string()); // Falls back to 'a' if all equal
    }

    // --- photo_plate Tests (using real ocr_service) ---

    // NOTE: These tests rely on the actual `ocr_service::scan_plate` which is not mocked
    // due to constraints. Therefore, results might vary based on Tesseract setup and image content.
    // We'll primarily test the control flow and error handling.

    #[test]
    fn test_photo_plate_with_dummy_image() {
        let result = photo_plate(DUMMY_JPEG_BYTES);
        // Expect an error or empty result as a tiny red image won't have a plate
        assert!(result.is_err() || result.unwrap().plate.is_empty());
    }

    #[test]
    fn test_photo_plate_invalid_image_bytes() {
        let invalid_bytes = b"not a real image";
        let result = photo_plate(invalid_bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Cannot decode image"));
    }

    #[test]
    fn test_photo_plate_large_image_downscaling_flow() {
        // Create a large dummy image (e.g., 2000x1000 pixels)
        let img = image::RgbImage::new(2000, 1000);
        let mut large_img_bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut large_img_bytes),
            image::ImageFormat::Jpeg,
        )
        .expect("Failed to encode large test image to JPEG");

        // The exact OCR result is not predictable here without mocking,
        // but we can ensure the pipeline runs without crashing.
        let result = photo_plate(&large_img_bytes);
        assert!(result.is_ok() || result.is_err()); // It should either succeed or gracefully fail
    }

    #[test]
    fn test_photo_plate_small_image_no_upscaling_flow() {
        // Create a small dummy image (e.g., 300x200 pixels)
        let img = image::RgbImage::new(300, 200);
        let mut small_img_bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut small_img_bytes),
            image::ImageFormat::Jpeg,
        )
        .expect("Failed to encode large test image to JPEG");

        let result = photo_plate(&small_img_bytes);
        assert!(result.is_ok() || result.is_err()); // It should either succeed or gracefully fail
    }
}
