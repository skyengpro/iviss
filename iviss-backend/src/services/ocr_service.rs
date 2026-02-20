use image::imageops::FilterType;
use image::GenericImageView;
use once_cell::sync::Lazy;
use regex::Regex;
use rusty_tesseract::{Args, Image};

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;

/// Cameroon plate format: 2 letters + 3 digits + 2 letters (e.g. CE128BC).
static PLATE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}[0-9]{3}[A-Z]{2}$").unwrap());

/// Target width (px) for the resized image fed to Tesseract.
const TARGET_WIDTH: u32 = 800;

/// Binarisation threshold (0–255). Pixels above this become white, below become black.
const BINARY_THRESHOLD: u8 = 128;

// ── public API ────────────────────────────────────────────────────────────────

/// Run the full OCR pipeline on raw image bytes (JPEG / PNG).
///
/// Pipeline: Load → Grayscale → Resize (800 px width) → Threshold → Tesseract.
///
/// Images are processed in-memory. `rusty-tesseract` writes a temp file
/// internally for the Tesseract CLI call, which is cleaned up automatically.
pub fn scan_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    // 1. Load the image from raw bytes
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;

    // 2. Convert to 8-bit grayscale
    let gray = img.to_luma8();

    // 3. Resize – preserve aspect ratio, target width = 800 px
    let (orig_w, _orig_h) = img.dimensions();
    let gray = if orig_w > TARGET_WIDTH {
        let scale = TARGET_WIDTH as f32 / orig_w as f32;
        let new_h = (img.height() as f32 * scale) as u32;
        image::imageops::resize(&gray, TARGET_WIDTH, new_h, FilterType::Lanczos3)
    } else {
        gray
    };

    // 4. Binarise (simple fixed threshold)
    let binary = image::GrayImage::from_fn(gray.width(), gray.height(), |x, y| {
        let px = gray.get_pixel(x, y);
        if px[0] >= BINARY_THRESHOLD {
            image::Luma([255u8])
        } else {
            image::Luma([0u8])
        }
    });

    // 5. Convert back to DynamicImage for rusty-tesseract
    let dynamic_img = image::DynamicImage::ImageLuma8(binary);

    // 6. Create a rusty-tesseract Image from the DynamicImage
    let tess_image = Image::from_dynamic_image(&dynamic_img)
        .map_err(|e| AppError::internal_error(format!("Failed to create Tesseract image: {e}")))?;

    // 7. Configure Tesseract arguments
    let args = Args {
        lang: "eng".to_string(),
        config_variables: HashMap::from([
            ("tessedit_char_whitelist".to_string(), "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string()),
        ]),
        dpi: Some(300),
        psm: Some(7), // Treat the image as a single text line
        oem: Some(3), // Default OCR engine mode
    };

    // 8. Run OCR
    let raw_text = rusty_tesseract::image_to_string(&tess_image, &args)
        .map_err(|e| AppError::internal_error(format!("Tesseract OCR failed: {e}")))?;

    // rusty-tesseract doesn't expose per-word confidence via the simple API,
    // so we use `image_to_data` to get word-level confidence scores.
    let confidence = get_mean_confidence(&tess_image, &args);

    // 9. Normalise the extracted text
    let normalised = normalise_plate(&raw_text);

    // 10. Validate against Cameroon plate format
    let format_valid = PLATE_REGEX.is_match(&normalised);

    Ok(ScanResultData {
        plate: if format_valid {
            normalised
        } else {
            String::new()
        },
        raw_text: raw_text.trim().to_string(),
        confidence,
        format_valid,
    })
}

// ── helpers ───────────────────────────────────────────────────────────────────

use std::collections::HashMap;

/// Get mean confidence from Tesseract data output.
/// Falls back to 0.0 if data extraction fails.
fn get_mean_confidence(image: &Image, args: &Args) -> f32 {
    match rusty_tesseract::image_to_data(image, args) {
        Ok(data) => {
            let confidences: Vec<f32> = data
                .data
                .iter()
                .filter(|d| d.conf > 0.0)
                .map(|d| d.conf)
                .collect();

            if confidences.is_empty() {
                0.0
            } else {
                let sum: f32 = confidences.iter().sum();
                (sum / confidences.len() as f32 / 100.0).clamp(0.0, 1.0)
            }
        }
        Err(_) => 0.0,
    }
}

/// Uppercase, strip spaces / punctuation, keep only alphanumeric chars.
fn normalise_plate(raw: &str) -> String {
    raw.to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!PLATE_REGEX.is_match("C128BC"));   // only 1 leading letter
        assert!(!PLATE_REGEX.is_match("CE12BC"));   // only 2 digits
        assert!(!PLATE_REGEX.is_match("CE1234BC")); // 4 digits
        assert!(!PLATE_REGEX.is_match("1E128BC"));  // starts with digit
        assert!(!PLATE_REGEX.is_match(""));
    }
}
