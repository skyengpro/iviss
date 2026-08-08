use std::time::Instant;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Rgb, RgbImage};

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
    let rectified = cropped.as_ref().map(perspective_rectify_color_crop);
    let source = rectified.as_ref().unwrap_or(&img);

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

/// Correct a small left/right perspective trapezoid on an already colour-cropped
/// plate.
fn perspective_rectify_color_crop(img: &DynamicImage) -> DynamicImage {
    let rgb = img.to_rgb8();
    let Some((source_quad, target_quad)) = estimate_plate_trapezoid(&rgb) else {
        return img.clone();
    };

    let Some(projection) = projection_from_quad(source_quad, target_quad) else {
        return img.clone();
    };

    let warped = imageproc::geometric_transformations::warp(
        &rgb,
        &projection,
        imageproc::geometric_transformations::Interpolation::Bilinear,
        Rgb([255, 255, 255]),
    );
    DynamicImage::ImageRgb8(warped)
}

/// Estimate only the side-edge slopes from the orange pixels. The top and
/// bottom edges remain untouched, which keeps this correction conservative.
type Quad = [(f32, f32); 4];

fn estimate_plate_trapezoid(rgb: &RgbImage) -> Option<(Quad, Quad)> {
    let (width, height) = rgb.dimensions();
    if width < 80 || height < 30 {
        return None;
    }

    let mut rows = Vec::new();
    for y in 0..height {
        let mut min_x = width;
        let mut max_x = 0;
        let mut count = 0;
        for x in 0..width {
            if is_orange_plate_pixel(rgb.get_pixel(x, y)) {
                count += 1;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
        }
        if count >= (width / 10).max(8) {
            rows.push((y as f32, min_x as f32, max_x as f32));
        }
    }

    let (top_y, bottom_y) = match (rows.first(), rows.last()) {
        (Some(first), Some(last)) if last.0 - first.0 >= height as f32 * 0.20 => (first.0, last.0),
        _ => return None,
    };

    let left_top = fit_edge(&rows, true, top_y)?;
    let left_bottom = fit_edge(&rows, true, bottom_y)?;
    let right_top = fit_edge(&rows, false, top_y)?;
    let right_bottom = fit_edge(&rows, false, bottom_y)?;
    let min_x = rows.iter().map(|row| row.1).fold(f32::INFINITY, f32::min);
    let max_x = rows
        .iter()
        .map(|row| row.2)
        .fold(f32::NEG_INFINITY, f32::max);
    let width = max_x - min_x;
    if width <= 0.0 {
        return None;
    }

    let drift = (left_bottom - left_top)
        .abs()
        .max((right_bottom - right_top).abs())
        / width;
    // Ignore noise and reject a strong perspective that this conservative
    // correction cannot represent safely.
    if !(0.02..=0.15).contains(&drift) {
        return None;
    }

    let source = [
        (left_top, top_y),
        (right_top, top_y),
        (right_bottom, bottom_y),
        (left_bottom, bottom_y),
    ];
    let target = [
        (min_x, top_y),
        (max_x, top_y),
        (max_x, bottom_y),
        (min_x, bottom_y),
    ];
    Some((source, target))
}

fn fit_edge(rows: &[(f32, f32, f32)], left: bool, y: f32) -> Option<f32> {
    let sum_y: f32 = rows.iter().map(|row| row.0).sum();
    let sum_x: f32 = rows
        .iter()
        .map(|row| if left { row.1 } else { row.2 })
        .sum();
    let mean_y = sum_y / rows.len() as f32;
    let mean_x = sum_x / rows.len() as f32;
    let denominator: f32 = rows.iter().map(|row| (row.0 - mean_y).powi(2)).sum();
    if denominator <= f32::EPSILON {
        return None;
    }
    let slope: f32 = rows
        .iter()
        .map(|row| (row.0 - mean_y) * ((if left { row.1 } else { row.2 }) - mean_x))
        .sum::<f32>()
        / denominator;
    Some(mean_x + slope * (y - mean_y))
}

fn is_orange_plate_pixel(pixel: &Rgb<u8>) -> bool {
    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    if delta <= 0.0 {
        return false;
    }
    let mut hue = if max == r {
        60.0 * ((g - b) / delta % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    if hue < 0.0 {
        hue += 360.0;
    }
    let saturation = delta / max;
    (10.0..=30.0).contains(&hue) && saturation >= 0.4 && max >= 0.4
}

fn projection_from_quad(
    source: [(f32, f32); 4],
    target: [(f32, f32); 4],
) -> Option<imageproc::geometric_transformations::Projection> {
    let mut equations = [[0.0f32; 9]; 8];
    for (index, ((x, y), (u, v))) in source.into_iter().zip(target).enumerate() {
        let row = index * 2;
        equations[row] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y, u];
        equations[row + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y, v];
    }

    for column in 0..8 {
        let pivot = (column..8).max_by(|&a, &b| {
            equations[a][column]
                .abs()
                .partial_cmp(&equations[b][column].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        if equations[pivot][column].abs() < 1e-6 {
            return None;
        }
        equations.swap(column, pivot);
        let divisor = equations[column][column];
        for value in equations[column].iter_mut().skip(column) {
            *value /= divisor;
        }
        let pivot_row = equations[column];
        for (row, equation) in equations.iter_mut().enumerate() {
            if row == column {
                continue;
            }
            let factor = equation[column];
            for (value, &pivot_value) in equation
                .iter_mut()
                .skip(column)
                .zip(pivot_row.iter().skip(column))
            {
                *value -= factor * pivot_value;
            }
        }
    }

    let mut matrix = [0.0f32; 9];
    matrix[..8].copy_from_slice(&equations.map(|row| row[8]));
    matrix[8] = 1.0;
    imageproc::geometric_transformations::Projection::from_matrix(matrix)
}

/// Second look at a scan result on the photo path.
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
