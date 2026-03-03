use std::io::Cursor;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;
use crate::services::ocr_service;

/// OCR pipeline for single-shot photo captures.
///
/// This service is intentionally separate from live scanning so it can evolve
/// independently (heavier preprocessing, different retry strategies, etc.).
pub fn photo_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    // Photo mode can afford a few heavier attempts. We still reuse the same
    // OCR engine + regex/selection logic from the scan pipeline.

    // Attempt 1: reuse the baseline scan OCR pipeline.
    let first = ocr_service::scan_plate(image_bytes)?;
    if first.format_valid {
        return Ok(first);
    }

    // Decode once; we will generate a few variants from the same decoded image.
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;

    let (w, h) = img.dimensions();

    // Keep aspect ratio; aim for a reasonable width for OCR.
    // We prefer downscaling huge images and upscaling small ones slightly.
    let target_w: u32 = if w >= 2000 { 1600 } else if w < 900 { 1200 } else { w };
    let target_h: u32 = (h as f64 * (target_w as f64 / w as f64)).round() as u32;

    let base_resized = img.resize(target_w, target_h, FilterType::CatmullRom);

    // Variants (bounded):
    // - resized (baseline)
    // - resized + contrast boost
    // - resized + mild unsharpen
    let v1 = run_variant("resized", &base_resized)?;
    if v1.format_valid {
        return Ok(v1);
    }

    let contrast = base_resized.adjust_contrast(30.0);
    let v2 = run_variant("resized-contrast", &contrast)?;
    if v2.format_valid {
        return Ok(v2);
    }

    let sharpened = base_resized.unsharpen(1.2, 1);
    let v3 = run_variant("resized-unsharpen", &sharpened)?;

    tracing::info!(
        "Photo OCR variants done: original={}x{} bytes={} target={}x{}",
        w,
        h,
        image_bytes.len(),
        target_w,
        target_h
    );

    Ok(pick_best_many(vec![first, v1, v2, v3]))
}

fn run_variant(label: &str, img: &DynamicImage) -> Result<ScanResultData, AppError> {
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| AppError::internal_error(format!("Failed to encode {label}: {e}")))?;

    let res = ocr_service::scan_plate(&buf)?;
    tracing::info!(
        "Photo OCR variant {label}: plate={:?} valid={} conf={:.2}",
        res.plate,
        res.format_valid,
        res.confidence
    );
    Ok(res)
}

fn pick_best_many(mut candidates: Vec<ScanResultData>) -> ScanResultData {
    // Prefer a valid plate, then confidence.
    candidates.sort_by(|a, b| {
        match (a.format_valid, b.format_valid) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b
                .confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    candidates.into_iter().next().unwrap_or_default()
}
