use std::io::Cursor;

use image::imageops::FilterType;
use image::GenericImageView;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;
use crate::services::ocr_service;

static PHOTO_PLATE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[A-Z]{2}[0-9]{3}[A-Z]{2}").unwrap());

/// OCR pipeline for single-shot photo captures.
///
/// This service is intentionally separate from live scanning so it can evolve
/// independently (heavier preprocessing, different retry strategies, etc.).
pub fn photo_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    // Photo OCR needs to feel snappy. We therefore cap work to *two* OCR runs:
    // 1) baseline scan pipeline
    // 2) one photo-specific preprocessed fallback

    let first = enhance_photo_result(ocr_service::scan_plate(image_bytes)?);
    if first.format_valid {
        return Ok(first);
    }

    // Fallback attempt: decode once, resize to a stable OCR-friendly size,
    // apply a small contrast boost + mild unsharpen, then re-run scan OCR.
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;

    let (w, h) = img.dimensions();

    // Keep aspect ratio; prefer downscaling large images.
    // (Upscaling rarely helps and can slow OCR.)
    let target_w: u32 = if w > 1200 { 1200 } else { w };
    let target_h: u32 = (h as f64 * (target_w as f64 / w as f64)).round() as u32;

    let resized = img.resize(target_w, target_h, FilterType::Triangle);
    let preprocessed = resized.adjust_contrast(25.0).unsharpen(1.0, 1);

    let mut buf: Vec<u8> = Vec::new();
    preprocessed
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
        .map_err(|e| AppError::internal_error(format!("Failed to encode fallback image: {e}")))?;

    tracing::info!(
        "Photo OCR fallback: {}x{} -> {}x{} ({} -> {} bytes)",
        w,
        h,
        target_w,
        target_h,
        image_bytes.len(),
        buf.len()
    );

    let second = enhance_photo_result(ocr_service::scan_plate(&buf)?);
    Ok(pick_best(first, second))
}

fn enhance_photo_result(mut r: ScanResultData) -> ScanResultData {
    // If the scan pipeline already extracted a plate, keep it (even if format is invalid).
    // Photo mode should still surface the best candidate to the client.
    if !r.plate.is_empty() {
        return r;
    }

    // If scan couldn't extract a plate, try a strict extraction from raw_text.
    // This mirrors the intent of the photo service without discarding useful OCR info.
    if let Some(p) = extract_plate_strict(&r.raw_text) {
        r.plate = p;
        r.format_valid = true;
        // Keep confidence semantics consistent with `ocr_service::finalize`, which
        // promotes valid-format plates to a high confidence score.
        if r.confidence < 0.90 {
            r.confidence = 0.90;
        }
    }

    r
}

fn extract_plate_strict(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    PHOTO_PLATE_REGEX
        .find(&cleaned)
        .map(|m| m.as_str().to_string())
}

fn pick_best(a: ScanResultData, b: ScanResultData) -> ScanResultData {
    // Priority 1: Valid format
    if a.format_valid && !b.format_valid {
        return a;
    }
    if b.format_valid && !a.format_valid {
        return b;
    }

    // Priority 2: If one has a candidate plate and the other does not, pick the one with text.
    if a.plate.is_empty() && !b.plate.is_empty() {
        return b;
    }
    if b.plate.is_empty() && !a.plate.is_empty() {
        return a;
    }

    // Priority 3: Higher confidence
    if b.confidence > a.confidence {
        return b;
    }

    a
}
