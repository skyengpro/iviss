use image::imageops::FilterType;
use image::{GenericImageView, GrayImage, Luma};
use once_cell::sync::Lazy;
use regex::Regex;
use rusty_tesseract::{Args, Image};
use std::collections::HashMap;

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;

/// Cameroon plate format: 2 letters + 3 digits + 2 letters (e.g. CE128BC).
static PLATE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}[0-9]{3}[A-Z]{2}$").unwrap());

/// Target width (px) for the resized image fed to Tesseract.
const TARGET_WIDTH: u32 = 800;

/// Radius (in pixels) for the adaptive threshold sliding window.
const ADAPTIVE_RADIUS: u32 = 15;

/// Offset subtracted from the local mean when applying adaptive threshold.
const ADAPTIVE_C: i16 = 10;

// ── public API ────────────────────────────────────────────────────────────────

/// Run the full OCR pipeline on raw image bytes (JPEG / PNG).
///
/// Pipeline:
///   Load → Grayscale → Resize → Contrast stretch → Adaptive threshold
///   → Tesseract (binary + inverted, PSM 7) → pick best result → normalise.
pub fn scan_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    // 1. Load the image from raw bytes
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;

    // 2. Convert to 8-bit grayscale
    let gray = img.to_luma8();

    // 3. Resize – scale to target width (also upscales small images)
    let (orig_w, _) = img.dimensions();
    let gray = if orig_w != TARGET_WIDTH {
        let scale = TARGET_WIDTH as f32 / orig_w as f32;
        let new_h = (img.height() as f32 * scale).max(1.0) as u32;
        image::imageops::resize(&gray, TARGET_WIDTH, new_h, FilterType::Lanczos3)
    } else {
        gray
    };

    // 4. Contrast stretch (min-max normalisation to full 0–255 range)
    let gray = contrast_stretch(&gray);

    // 5. Adaptive threshold (handles uneven lighting / coloured backgrounds)
    let binary = adaptive_threshold(&gray, ADAPTIVE_RADIUS, ADAPTIVE_C);

    // 6. Also produce an inverted version (handles dark text on light plates
    //    vs light text on dark/coloured plates like orange Cameroon plates).
    let inverted = invert_image(&binary);

    // 7. Tesseract args — single text line (PSM 7), A-Z + 0-9 whitelist
    let args = Args {
        lang: "eng".to_string(),
        config_variables: HashMap::from([(
            "tessedit_char_whitelist".to_string(),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string(),
        )]),
        dpi: Some(300),
        psm: Some(7),
        oem: Some(3),
    };

    // 8. Run OCR on binary image
    let result_binary = try_ocr(&binary, &args);

    // If the binary variant already matched the plate regex, return immediately
    if let Some(ref r) = result_binary {
        if r.format_valid {
            return Ok(r.clone());
        }
    }

    // 9. Run OCR on inverted image
    let result_inverted = try_ocr(&inverted, &args);

    if let Some(ref r) = result_inverted {
        if r.format_valid {
            return Ok(r.clone());
        }
    }

    // 10. Neither matched — return whichever had more text / higher confidence
    Ok(pick_best(result_binary, result_inverted))
}

// ── OCR helper ────────────────────────────────────────────────────────────────

/// Attempt OCR on a single grayscale image. Returns None if Tesseract fails.
fn try_ocr(img: &GrayImage, args: &Args) -> Option<ScanResultData> {
    let dynamic_img = image::DynamicImage::ImageLuma8(img.clone());
    let tess_image = Image::from_dynamic_image(&dynamic_img).ok()?;

    let raw_text = rusty_tesseract::image_to_string(&tess_image, args).ok()?;
    let confidence = get_mean_confidence(&tess_image, args);
    let normalised = normalise_plate(&raw_text);
    let format_valid = PLATE_REGEX.is_match(&normalised);

    Some(ScanResultData {
        plate: if format_valid { normalised } else { String::new() },
        raw_text: raw_text.trim().to_string(),
        confidence,
        format_valid,
    })
}

/// Pick the better of two optional OCR results.
fn pick_best(a: Option<ScanResultData>, b: Option<ScanResultData>) -> ScanResultData {
    let empty = ScanResultData {
        plate: String::new(),
        raw_text: String::new(),
        confidence: 0.0,
        format_valid: false,
    };

    match (a, b) {
        (Some(a), Some(b)) => {
            if a.format_valid { a }
            else if b.format_valid { b }
            else if a.confidence >= b.confidence && !a.raw_text.is_empty() { a }
            else if !b.raw_text.is_empty() { b }
            else { a }
        }
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => empty,
    }
}

// ── image processing helpers ──────────────────────────────────────────────────

/// Min-max contrast stretch: maps the pixel range [min, max] → [0, 255].
fn contrast_stretch(img: &GrayImage) -> GrayImage {
    let mut min_val = 255u8;
    let mut max_val = 0u8;
    for px in img.pixels() {
        min_val = min_val.min(px[0]);
        max_val = max_val.max(px[0]);
    }
    if max_val == min_val {
        return img.clone();
    }
    let range = (max_val - min_val) as f32;
    GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let v = img.get_pixel(x, y)[0];
        Luma([((v as f32 - min_val as f32) / range * 255.0) as u8])
    })
}

/// Adaptive thresholding using a local mean (integral-image based, O(1) per pixel).
fn adaptive_threshold(img: &GrayImage, radius: u32, c: i16) -> GrayImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let iw = w + 1;

    // Build integral image
    let mut integral = vec![0i64; iw * (h + 1)];
    for y in 0..h {
        let mut row_sum = 0i64;
        for x in 0..w {
            row_sum += img.get_pixel(x as u32, y as u32)[0] as i64;
            integral[(y + 1) * iw + (x + 1)] = row_sum + integral[y * iw + (x + 1)];
        }
    }

    let r = radius as usize;
    GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let (xi, yi) = (x as usize, y as usize);
        let x1 = xi.saturating_sub(r);
        let y1 = yi.saturating_sub(r);
        let x2 = (xi + r + 1).min(w);
        let y2 = (yi + r + 1).min(h);
        let count = ((x2 - x1) * (y2 - y1)) as i64;
        let sum = integral[y2 * iw + x2] - integral[y1 * iw + x2]
            - integral[y2 * iw + x1] + integral[y1 * iw + x1];
        let threshold = ((sum / count) as i16 - c).max(0) as u8;
        if img.get_pixel(x, y)[0] > threshold { Luma([255u8]) } else { Luma([0u8]) }
    })
}

/// Invert a grayscale image: 255 → 0, 0 → 255.
fn invert_image(img: &GrayImage) -> GrayImage {
    GrayImage::from_fn(img.width(), img.height(), |x, y| {
        Luma([255 - img.get_pixel(x, y)[0]])
    })
}

/// Get mean confidence from Tesseract data output. Falls back to 0.0.
fn get_mean_confidence(image: &Image, args: &Args) -> f32 {
    match rusty_tesseract::image_to_data(image, args) {
        Ok(data) => {
            let confs: Vec<f32> = data.data.iter()
                .filter(|d| d.conf > 0.0)
                .map(|d| d.conf)
                .collect();
            if confs.is_empty() { 0.0 }
            else { (confs.iter().sum::<f32>() / confs.len() as f32 / 100.0).clamp(0.0, 1.0) }
        }
        Err(_) => 0.0,
    }
}

/// Uppercase, strip non-alphanumeric chars.
fn normalise_plate(raw: &str) -> String {
    raw.to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect()
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
        assert!(!PLATE_REGEX.is_match("C128BC"));
        assert!(!PLATE_REGEX.is_match("CE12BC"));
        assert!(!PLATE_REGEX.is_match("CE1234BC"));
        assert!(!PLATE_REGEX.is_match("1E128BC"));
        assert!(!PLATE_REGEX.is_match(""));
    }

    #[test]
    fn contrast_stretch_full_range() {
        let img = GrayImage::from_fn(3, 1, |x, _| {
            Luma([match x { 0 => 100, 1 => 150, _ => 200 }])
        });
        let stretched = contrast_stretch(&img);
        assert_eq!(stretched.get_pixel(0, 0)[0], 0);
        assert_eq!(stretched.get_pixel(2, 0)[0], 255);
    }

    #[test]
    fn adaptive_threshold_basic() {
        let img = GrayImage::from_fn(5, 1, |x, _| {
            Luma([if x == 2 { 200 } else { 10 }])
        });
        let result = adaptive_threshold(&img, 2, 5);
        assert_eq!(result.get_pixel(2, 0)[0], 255);
    }
}
