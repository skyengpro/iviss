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
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;

    // 1. Color-adaptive crop (isolates the plate region based on known color profiles)
    let cropped = color_adaptive_crop(&img).unwrap_or_else(|| img.clone());
    
    // 2. Smart upscaling for small crops (ensures characters are at least ~30px tall)
    let (cw, ch) = cropped.dimensions();
    let base_img = if cw < 400 {
        let scale = 400.0 / cw as f32;
        cropped.resize(400, (ch as f32 * scale) as u32, FilterType::Triangle)
    } else {
        cropped.clone()
    };

    let run_ocr = |i: &image::DynamicImage| -> Result<ScanResultData, AppError> {
        let mut buf: Vec<u8> = Vec::new();
        i.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .map_err(|e| AppError::internal_error(format!("Failed to encode image: {e}")))?;
        Ok(enhance_photo_result(ocr_service::scan_plate(&buf)?))
    };

    // 3. Multi-scale OCR: Try 1x scale
    let first = run_ocr(&base_img)?;
    if first.format_valid {
        return Ok(first);
    }

    // Fallback: 1.5x upscale with contrast boost and unsharpening
    let (bw, bh) = base_img.dimensions();
    let upscale_img = base_img
        .resize((bw as f32 * 1.5) as u32, (bh as f32 * 1.5) as u32, FilterType::Triangle)
        .adjust_contrast(25.0)
        .unsharpen(1.0, 1);
        
    let second = run_ocr(&upscale_img)?;
    Ok(pick_best(first, second))
}

fn color_adaptive_crop(img: &image::DynamicImage) -> Option<image::DynamicImage> {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    
    // HSV profiles for Cameroon plates: (Name, Box<dyn Fn(H,S,V) -> bool>)
    let profiles: &[(&str, fn(f32, f32, f32) -> bool)] = &[
        ("orange", |h, s, v| h >= 10.0 && h <= 30.0 && s >= 0.4 && v >= 0.4),
        ("yellow", |h, s, v| h >= 20.0 && h <= 50.0 && s >= 0.3 && v >= 0.5),
        ("white", |_, s, v| s <= 0.15 && v >= 0.8),
        ("red", |h, s, v| (h <= 10.0 || h >= 340.0) && s >= 0.4 && v >= 0.3),
        ("green", |h, s, v| h >= 35.0 && h <= 85.0 && s >= 0.2 && v >= 0.2),
    ];
    
    for (name, condition) in profiles {
        let mut mask = image::GrayImage::new(w, h);
        let mut match_count = 0;
        
        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        let mut max_x = 0;
        let mut max_y = 0;
        
        for (x, y, p) in rgb.enumerate_pixels() {
            let r = p[0] as f32 / 255.0;
            let g = p[1] as f32 / 255.0;
            let b = p[2] as f32 / 255.0;
            
            let cmax = r.max(g).max(b);
            let cmin = r.min(g).min(b);
            let delta = cmax - cmin;
            
            let mut hue = 0.0;
            if delta > 0.0 {
                if cmax == r {
                    hue = 60.0 * (((g - b) / delta) % 6.0);
                } else if cmax == g {
                    hue = 60.0 * (((b - r) / delta) + 2.0);
                } else {
                    hue = 60.0 * (((r - g) / delta) + 4.0);
                }
            }
            if hue < 0.0 { hue += 360.0; }
            
            let s = if cmax == 0.0 { 0.0 } else { delta / cmax };
            let v = cmax;
            
            if condition(hue, s, v) {
                mask.put_pixel(x, y, image::Luma([255]));
                match_count += 1;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
        
        // If matched region is at least 1% of the image
        if match_count > (w * h) / 100 && min_x <= max_x && min_y <= max_y {
            let cw = max_x - min_x + 1;
            let ch = max_y - min_y + 1;
            let aspect = cw as f32 / ch as f32;
            
            // Standard plates are ~4.7:1, two-line plates are ~2:1. Allow 1.5 to 7.0.
            if aspect >= 1.5 && aspect <= 7.0 {
                tracing::info!("Color profile match: {} (aspect {:.1})", name, aspect);
                // Add 10% padding
                let pad_w = (cw as f32 * 0.1) as u32;
                let pad_h = (ch as f32 * 0.1) as u32;
                let cx = min_x.saturating_sub(pad_w);
                let cy = min_y.saturating_sub(pad_h);
                let ccw = (cw + 2 * pad_w).min(w - cx);
                let cch = (ch + 2 * pad_h).min(h - cy);
                return Some(img.crop_imm(cx, cy, ccw, cch));
            }
        }
    }
    None
}

pub(crate) fn enhance_photo_result(mut r: ScanResultData) -> ScanResultData {
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

pub(crate) fn extract_plate_strict(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    PHOTO_PLATE_REGEX
        .find(&cleaned)
        .map(|m| m.as_str().to_string())
}

pub(crate) fn pick_best(a: ScanResultData, b: ScanResultData) -> ScanResultData {
    // Priority 1: Valid format
    if a.format_valid && !b.format_valid {
        return a;
    }
    if b.format_valid && !a.format_valid {
        return b;
    }

    // Character-level voting if both are invalid but have the same length
    if !a.plate.is_empty() && !b.plate.is_empty() && a.plate.len() == b.plate.len() {
        let mut voted = String::new();
        for (ca, cb) in a.plate.chars().zip(b.plate.chars()) {
            if ca == cb {
                voted.push(ca);
            } else {
                // Break tie with confidence
                voted.push(if a.confidence > b.confidence { ca } else { cb });
            }
        }
        let mut res = if a.confidence > b.confidence { a.clone() } else { b.clone() };
        res.plate = voted;
        if PHOTO_PLATE_REGEX.is_match(&res.plate) {
            res.format_valid = true;
            res.confidence = 0.90;
        }
        return res;
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
