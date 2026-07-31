use std::time::Instant;

use image::imageops::FilterType;
use image::GenericImageView;

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;
use crate::services::ocr_service;
use crate::services::ocr_timings::{OcrBudget, Stage, StageTimings};
use crate::utils::plate_format;

/// A crop at or above this width/height ratio is already plate-shaped: the
/// frontend viewfinder framed it, and the colour crop can only degrade it.
const PLATE_SHAPED_ASPECT: f32 = 3.0;

/// Crops narrower than this are upscaled before OCR.
const MIN_OCR_WIDTH: u32 = 400;

/// Scale factor of the enhanced retry pass.
const ENHANCED_PASS_SCALE: f32 = 1.5;

/// OCR pipeline for single-shot photo captures.
///
/// This service is intentionally separate from live scanning so it can evolve
/// independently (heavier preprocessing, different retry strategies, etc.).
pub fn photo_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    let started = Instant::now();
    let budget = OcrBudget::new(ocr_service::OCR_STAGE_BUDGET);
    let mut timings = StageTimings::default();

    let img = ocr_service::decode_image(image_bytes, &mut timings)?;

    // 1. Colour-adaptive crop, skipped when the frontend already sent a
    //    plate-shaped crop: the orange profile also catches skin, wood and
    //    earth, so on a tight crop it has nothing to gain and something to lose.
    let cropped = timings.time(Stage::Crop, || {
        let (w, h) = img.dimensions();
        let plate_shaped = h > 0 && (w as f32 / h as f32) >= PLATE_SHAPED_ASPECT;
        if plate_shaped {
            None
        } else {
            color_adaptive_crop(&img)
        }
    });
    let source = cropped.as_ref().unwrap_or(&img);

    // 2. Smart upscaling for small crops (keeps characters reasonably tall)
    let (cw, ch) = source.dimensions();
    let upscaled = (cw > 0 && cw < MIN_OCR_WIDTH).then(|| {
        let scale = MIN_OCR_WIDTH as f32 / cw as f32;
        source.resize(
            MIN_OCR_WIDTH,
            (ch as f32 * scale) as u32,
            FilterType::Triangle,
        )
    });
    let base_img = upscaled.as_ref().unwrap_or(source);

    // 3. Native pass. The decoded image goes straight to the scan pipeline —
    //    no JPEG round-trip, which would throw away detail right before
    //    recognition.
    let first = enhance_photo_result(ocr_service::scan_plate_image(
        base_img,
        &mut timings,
        &budget,
    )?);

    // Retry only when the native pass read no text at all. An invalid format is
    // not evidence that a heavier pass would do better; it just doubles the cost.
    if !first.raw_text.trim().is_empty() {
        timings.total = started.elapsed();
        timings.emit("photo");
        return ocr_service::finalize(first, &timings);
    }

    let (bw, bh) = base_img.dimensions();
    let enhanced = base_img
        .resize(
            (bw as f32 * ENHANCED_PASS_SCALE) as u32,
            (bh as f32 * ENHANCED_PASS_SCALE) as u32,
            FilterType::Triangle,
        )
        .adjust_contrast(25.0)
        .unsharpen(1.0, 1);

    let second = enhance_photo_result(ocr_service::scan_plate_image(
        &enhanced,
        &mut timings,
        &budget,
    )?);

    timings.total = started.elapsed();
    timings.emit("photo");
    ocr_service::finalize(pick_best(first, second), &timings)
}

fn color_adaptive_crop(img: &image::DynamicImage) -> Option<image::DynamicImage> {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    type ColorProfile = fn(f32, f32, f32) -> bool;
    // HSV profiles for Cameroon plates: (Name, Box<dyn Fn(H,S,V) -> bool>)
    let profiles: &[(&str, ColorProfile)] = &[
        ("orange", |h, s, v| {
            (10.0..=30.0).contains(&h) && s >= 0.4 && v >= 0.4
        }),
        ("yellow", |h, s, v| {
            (20.0..=50.0).contains(&h) && s >= 0.3 && v >= 0.5
        }),
        ("white", |_, s, v| s <= 0.15 && v >= 0.8),
        ("red", |h, s, v| {
            (h <= 10.0 || h >= 340.0) && s >= 0.4 && v >= 0.3
        }),
        ("green", |h, s, v| {
            (35.0..=85.0).contains(&h) && s >= 0.2 && v >= 0.2
        }),
    ];

    for (name, condition) in profiles {
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
            if hue < 0.0 {
                hue += 360.0;
            }

            let s = if cmax == 0.0 { 0.0 } else { delta / cmax };
            let v = cmax;

            if condition(hue, s, v) {
                match_count += 1;
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }

        // If matched region is at least 1% of the image
        if match_count > (w * h) / 100 && min_x <= max_x && min_y <= max_y {
            let cw = max_x - min_x + 1;
            let ch = max_y - min_y + 1;
            let aspect = cw as f32 / ch as f32;

            // Standard plates are ~4.7:1, two-line plates are ~2:1. Allow 1.5 to 7.0.
            if (1.5..=7.0).contains(&aspect) {
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

/// Second look at a scan result on the photo path.
///
/// It never touches `confidence`: that stays the raw Tesseract measurement, on
/// this path as on the scan path. Flooring it here would leave any client-side
/// threshold calibration inoperative for every photo capture, since every
/// `photo_plate` call goes through this function.
pub(crate) fn enhance_photo_result(mut r: ScanResultData) -> ScanResultData {
    // If the scan pipeline already extracted a plate, keep it (even if format is invalid).
    // Photo mode should still surface the best candidate to the client.
    if !r.plate.is_empty() {
        if let Some(found) = plate_format::classify(&r.plate) {
            r.plate = found.plate;
            r.format_valid = true;
            r.plate_type = Some(found.category.as_str().to_string());
        }
        return r;
    }

    // If scan couldn't extract a plate, try a strict extraction from raw_text.
    // This mirrors the intent of the photo service without discarding useful OCR info.
    if let Some(p) = extract_plate_strict(&r.raw_text) {
        // `extract_plate_strict` cannot guarantee the extracted string
        // classifies — that invariant belongs to another module — so derive the
        // flag instead of asserting it.
        let plate_match = plate_format::classify(&p);
        r.format_valid = plate_match.is_some();
        r.plate_type = plate_match.map(|m| m.category.as_str().to_string());
        r.plate = p;
    }

    r
}

pub(crate) fn extract_plate_strict(raw: &str) -> Option<String> {
    plate_format::extract_first(raw).map(|m| m.plate)
}

/// Select between two readings.
///
/// Selection only ever returns a reading exactly as it was recognised. The
/// former character-level vote built a third string from the two — a plate no
/// pass actually read — and then promoted it to `format_valid` whenever the
/// composite happened to match a pattern.
pub(crate) fn pick_best(a: ScanResultData, b: ScanResultData) -> ScanResultData {
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
