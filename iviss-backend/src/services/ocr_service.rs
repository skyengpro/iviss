use image::{GenericImageView, GrayImage};
use leptess::{LepTess, Variable};
use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;

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

pub struct TesseractGuard {
    tess: Option<LepTess>,
}

impl TesseractGuard {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            tess: Some(take_tesseract()?),
        })
    }
}

impl Deref for TesseractGuard {
    type Target = LepTess;
    fn deref(&self) -> &Self::Target {
        self.tess.as_ref().expect("tesseract must be present")
    }
}

impl DerefMut for TesseractGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tess.as_mut().expect("tesseract must be present")
    }
}

impl Drop for TesseractGuard {
    fn drop(&mut self) {
        if let Some(tess) = self.tess.take() {
            put_tesseract(tess);
        }
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Run the full OCR pipeline on raw image bytes (JPEG / PNG).
pub fn scan_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    let start_total = std::time::Instant::now();

    // 1. Load the image
    let load_start = std::time::Instant::now();
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;
    let load_elapsed = load_start.elapsed();

    let (width, height) = img.dimensions();
    tracing::info!(
        "Received image for OCR: {}x{} ({} bytes), load took {:?}",
        width,
        height,
        image_bytes.len(),
        load_elapsed
    );

    // 2. Convert to 8-bit grayscale
    let gray = img.to_luma8();

    // 3. Preprocessing: deskew → percentile contrast stretch → adaptive threshold → morphology
    let process_start = std::time::Instant::now();

    // Rotation correction
    let deskewed = deskew(&gray);

    // Contrast stretch (using 2nd/98th percentiles)
    let stretched = contrast_stretch_percentile(&deskewed);

    // Local adaptive threshold
    let binary_raw = adaptive_threshold(&stretched, ADAPTIVE_RADIUS, ADAPTIVE_C);

    // Morphological opening to clean up noise (small white blobs in black regions)
    let binary_clean = morphology_open(&binary_raw);

    // Add 30px white border — Tesseract works much better when chars aren't touching edges
    let binary = add_border(&binary_clean, 30, 255);

    // Lazily computed; only needed if binary variants fail.
    let mut inverted: Option<GrayImage> = None;

    let process_elapsed = process_start.elapsed();

    // 4. Initialize / reuse Tesseract
    let tess_init_start = std::time::Instant::now();
    let mut tess = TesseractGuard::new()?;
    let tess_init_elapsed = tess_init_start.elapsed();

    let tesseract_start = std::time::Instant::now();

    // Write the preprocessed image(s) once per request and reuse across OCR passes.
    // leptonica/leptess works most reliably via file paths.
    let tmp_id = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let bin_path = format!("/tmp/ocr_bin_{tmp_id}.png");
    binary
        .save(&bin_path)
        .map_err(|e| AppError::internal_error(format!("Failed to write temp image: {e}")))?;
    let mut inv_path: Option<String> = None;

    // --- MODE 1: PSM 7 (Single Text Line) ---
    tess.set_variable(Variable::TesseditPagesegMode, "7")
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM 7: {e}")))?;
    let r_b7 = try_ocr_path(&mut tess, &bin_path, "binary-psm7");

    if let Some(ref res) = r_b7 {
        if res.format_valid {
            return finalize(
                res.clone(),
                process_elapsed,
                tesseract_start.elapsed(),
                start_total.elapsed(),
            );
        }
    }

    let r_i7 = {
        if inverted.is_none() {
            inverted = Some(add_border(&invert_image(&binary), 0, 255));
            let p = format!("/tmp/ocr_inv_{tmp_id}.png");
            inverted.as_ref().unwrap().save(&p).map_err(|e| {
                AppError::internal_error(format!("Failed to write temp image: {e}"))
            })?;
            inv_path = Some(p);
        }
        inv_path
            .as_ref()
            .and_then(|p| try_ocr_path(&mut tess, p, "inverted-psm7"))
    };

    if let Some(ref res) = r_i7 {
        if res.format_valid {
            return finalize(
                res.clone(),
                process_elapsed,
                tesseract_start.elapsed(),
                start_total.elapsed(),
            );
        }
    }

    // --- MODE 2: PSM 8 (Single Word) ---
    tess.set_variable(Variable::TesseditPagesegMode, "8")
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM 8: {e}")))?;
    let r_b8 = try_ocr_path(&mut tess, &bin_path, "binary-psm8");

    if let Some(ref res) = r_b8 {
        if res.format_valid {
            return finalize(
                res.clone(),
                process_elapsed,
                tesseract_start.elapsed(),
                start_total.elapsed(),
            );
        }
    }

    // --- MODE 3: PSM 11 (Sparse Text) --- Fallback
    tess.set_variable(Variable::TesseditPagesegMode, "11")
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM 11: {e}")))?;
    let r_b11 = try_ocr_path(&mut tess, &bin_path, "binary-psm11");

    if let Some(ref res) = r_b11 {
        if res.format_valid {
            return finalize(
                res.clone(),
                process_elapsed,
                tesseract_start.elapsed(),
                start_total.elapsed(),
            );
        }
    }

    let r_i11 = {
        if inverted.is_none() {
            inverted = Some(add_border(&invert_image(&binary), 0, 255));
            let p = format!("/tmp/ocr_inv_{tmp_id}.png");
            inverted.as_ref().unwrap().save(&p).map_err(|e| {
                AppError::internal_error(format!("Failed to write temp image: {e}"))
            })?;
            inv_path = Some(p);
        }
        inv_path
            .as_ref()
            .and_then(|p| try_ocr_path(&mut tess, p, "inverted-psm11"))
    };

    // --- MODE 4: PSM 13 (Raw Line) --- Fallback
    tess.set_variable(Variable::TesseditPagesegMode, "13")
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM 13: {e}")))?;
    let r_b13 = try_ocr_path(&mut tess, &bin_path, "binary-psm13");

    let tesseract_elapsed = tesseract_start.elapsed();

    // 5. Result Selection (Voting via pick_best_ensemble)
    let candidates = vec![r_b7, r_i7, r_b8, r_b11, r_i11, r_b13];
    let final_result = pick_best_ensemble(candidates);

    // Best-effort cleanup of temp files
    let _ = std::fs::remove_file(&bin_path);
    if let Some(p) = inv_path.as_ref() {
        let _ = std::fs::remove_file(p);
    }

    // Add tesseract init time into logs by folding it into process_elapsed.
    // (We don't change the response; this is purely for observability.)
    let _ = tess_init_elapsed;
    finalize(
        final_result,
        process_elapsed,
        tesseract_elapsed,
        start_total.elapsed(),
    )
}

pub fn finalize(
    mut res: ScanResultData,
    proc: std::time::Duration,
    tess: std::time::Duration,
    total: std::time::Duration,
) -> Result<ScanResultData, AppError> {
    if res.format_valid {
        res.confidence = 0.90;
    } else if !res.plate.is_empty() {
        res.confidence = 0.50;
    }

    tracing::info!(
        "Scan completed: process={:?}, tesseract={:?}, total={:?}, plate={:?} (conf={:.2})",
        proc,
        tess,
        total,
        res.plate,
        res.confidence
    );

    Ok(res)
}

// ── OCR helper ────────────────────────────────────────────────────────────────

pub fn take_tesseract() -> Result<LepTess, AppError> {
    TESSERACT.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            let mut tess = LepTess::new(Some("/usr/share/tesseract-ocr/5/tessdata"), "eng")
                .map_err(|e| AppError::internal_error(format!("Failed to init Tesseract: {e}")))?;

            tess.set_variable(
                Variable::TesseditCharWhitelist,
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            )
            .map_err(|e| AppError::internal_error(format!("Failed to set whitelist: {e}")))?;

            *slot = Some(tess);
        }

        Ok(slot.take().expect("Tesseract slot must be initialized"))
    })
}

pub fn put_tesseract(tess: LepTess) {
    TESSERACT.with(|cell| {
        *cell.borrow_mut() = Some(tess);
    });
}

/// Attempt OCR on a single image path using leptess.
pub fn try_ocr_path(tess: &mut LepTess, img_path: &str, label: &str) -> Option<ScanResultData> {
    tess.set_image(img_path).ok()?;
    tess.set_source_resolution(300);

    let raw_text = tess.get_utf8_text().unwrap_or_default();
    let trimmed = raw_text.trim();
    let confidence = tess.mean_text_conf() as f32 / 100.0;
    let extracted = extract_plate_fuzzy(trimmed);
    let format_valid = extracted
        .as_ref()
        .map(|p| PLATE_REGEX.is_match(p))
        .unwrap_or(false);

    tracing::info!(
        "[{}] OCR raw: {:?} (conf: {:.2}), extracted: {:?}, valid: {}",
        label,
        trimmed,
        confidence,
        extracted,
        format_valid
    );

    if trimmed.is_empty() {
        return None;
    }

    Some(ScanResultData {
        plate: extracted.unwrap_or_default(),
        raw_text: trimmed.to_string(),
        confidence,
        format_valid,
    })
}

/// Pick the best result from an ensemble of candidates.
pub fn pick_best_ensemble(candidates: Vec<Option<ScanResultData>>) -> ScanResultData {
    let mut best: Option<ScanResultData> = None;

    for cand in candidates.into_iter().flatten() {
        match &best {
            None => best = Some(cand),
            Some(curr) => {
                let better =
                    (cand.format_valid, cand.confidence) > (curr.format_valid, curr.confidence);
                if better {
                    best = Some(cand);
                }
            }
        }
    }

    best.unwrap_or(ScanResultData {
        plate: String::new(),
        raw_text: String::new(),
        confidence: 0.0,
        format_valid: false,
    })
}

// ── image processing helpers ──────────────────────────────────────────────────

/// Percentile-based contrast stretch: maps the pixel range [p2, p98] → [0, 255].
/// This ignores extreme outlier pixels (like sun glare or deep shadows).
pub fn contrast_stretch_percentile(img: &GrayImage) -> GrayImage {
    let pixels = img.as_raw();
    if pixels.is_empty() {
        return img.clone();
    }

    let mut histogram = [0u64; 256];
    for &px in pixels {
        histogram[px as usize] += 1;
    }

    let total_pixels = pixels.len() as u64;
    let drop_count = (total_pixels as f32 * 0.02) as u64;

    let mut min_val = 0u8;
    let mut count = 0u64;
    for (i, &freq) in histogram.iter().enumerate() {
        count += freq;
        if count > drop_count {
            min_val = i as u8;
            break;
        }
    }

    let mut max_val = 255u8;
    let mut count_rev = 0u64;
    for (i, &freq) in histogram.iter().enumerate().rev() {
        count_rev += freq;
        if count_rev > drop_count {
            max_val = i as u8;
            break;
        }
    }

    if max_val <= min_val {
        return img.clone();
    }

    let range = (max_val - min_val) as f32;
    let (w, h) = img.dimensions();
    let mut out = GrayImage::new(w, h);

    // Predetermine lookup table for speed
    let mut lut = [0u8; 256];
    for (i, v) in lut.iter_mut().enumerate() {
        let mut val = (i as f32 - min_val as f32) / range * 255.0;
        val = val.clamp(0.0, 255.0);
        *v = val as u8;
    }

    for (out_px, &in_px) in out.iter_mut().zip(pixels.iter()) {
        *out_px = lut[in_px as usize];
    }

    out
}

/// Deskew (rotation correction) using horizontal projection profile variance.
/// Evaluates angles between -7 and +7 degrees to find the sharpest horizontal alignment.
pub fn deskew(img: &GrayImage) -> GrayImage {
    use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
    let (w, h) = img.dimensions();

    let mut best_angle = 0.0;
    let mut max_variance = 0.0;

    for angle_deg in -7..=7 {
        let rad = (angle_deg as f32).to_radians();
        let rotated = rotate_about_center(img, rad, Interpolation::Bilinear, image::Luma([0]));

        let mut row_sums = vec![0u32; h as usize];
        for y in 0..h {
            for x in 0..w {
                row_sums[y as usize] += rotated.get_pixel(x, y)[0] as u32;
            }
        }

        let mean = row_sums.iter().sum::<u32>() as f32 / h as f32;
        let mut variance = 0.0;
        for &sum in &row_sums {
            let diff = sum as f32 - mean;
            variance += diff * diff;
        }

        if variance > max_variance {
            max_variance = variance;
            best_angle = rad;
        }
    }

    if best_angle.abs() > 0.01 {
        rotate_about_center(img, best_angle, Interpolation::Bilinear, image::Luma([0]))
    } else {
        img.clone()
    }
}

/// Simple 3x3 morphological opening (erosion followed by dilation).
/// Cleans up small noise specs in the binary image.
pub fn morphology_open(img: &GrayImage) -> GrayImage {
    let (w, h) = img.dimensions();
    if w < 3 || h < 3 {
        return img.clone();
    }

    // Erode (initialize with input to preserve borders)
    let mut eroded = img.clone();
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let mut min_val = 255;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = img.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)[0];
                    if px < min_val {
                        min_val = px;
                    }
                }
            }
            eroded.put_pixel(x, y, image::Luma([min_val]));
        }
    }

    // Dilate (initialize with eroded to preserve borders)
    let mut out = eroded.clone();
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let mut max_val = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = eroded.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)[0];
                    if px > max_val {
                        max_val = px;
                    }
                }
            }
            out.put_pixel(x, y, image::Luma([max_val]));
        }
    }
    out
}

/// Adaptive thresholding using a local mean (integral-image based, O(1) per pixel).
pub fn adaptive_threshold(img: &GrayImage, radius: u32, c: i16) -> GrayImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let iw = w + 1;
    let pixels = img.as_raw();

    // Build integral image
    let mut integral = vec![0i64; iw * (h + 1)];
    for y in 0..h {
        let mut row_sum = 0i64;
        let row_off = y * w;
        let int_curr_row_off = (y + 1) * iw;
        let int_prev_row_off = y * iw;

        for x in 0..w {
            row_sum += pixels[row_off + x] as i64;
            integral[int_curr_row_off + (x + 1)] = row_sum + integral[int_prev_row_off + (x + 1)];
        }
    }

    let r = radius as usize;
    let mut out = GrayImage::new(img.width(), img.height());
    let out_pixels = out.as_mut();

    for y in 0..h {
        let y1 = y.saturating_sub(r);
        let y2 = (y + r + 1).min(h);
        let row_off = y * w;

        for x in 0..w {
            let x1 = x.saturating_sub(r);
            let x2 = (x + r + 1).min(w);

            let count = ((x2 - x1) * (y2 - y1)) as i64;
            let sum = integral[y2 * iw + x2] - integral[y1 * iw + x2] - integral[y2 * iw + x1]
                + integral[y1 * iw + x1];

            let threshold = ((sum / count) as i16 - c).max(0) as u8;
            if pixels[row_off + x] > threshold {
                out_pixels[row_off + x] = 255;
            } else {
                out_pixels[row_off + x] = 0;
            }
        }
    }
    out
}

/// Invert a grayscale image: 255 → 0, 0 → 255.
pub fn invert_image(img: &GrayImage) -> GrayImage {
    let mut out = img.clone();
    for px in out.iter_mut() {
        *px = 255 - *px;
    }
    out
}

/// Add a solid color border around the image.
pub fn add_border(img: &GrayImage, border: u32, color: u8) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut out = GrayImage::from_pixel(w + 2 * border, h + 2 * border, image::Luma([color]));
    image::imageops::replace(&mut out, img, border as i64, border as i64);
    out
}

/// Cameroon plate: 2 letters + 3 digits + 2 letters.
/// Apply position-aware correction for common OCR misreads.
pub fn extract_plate_fuzzy(raw: &str) -> Option<String> {
    let cleaned = normalise_plate(raw);

    // 1. First priority: find a sequence that matches the Cameroon format exactly
    if let Some(mat) = PLATE_REGEX.find(&cleaned) {
        return Some(mat.as_str().to_string());
    }

    // 2. Second priority: find a 7-character sequence and try to correct it
    // LL DDD LL
    if cleaned.len() >= 7 {
        // Find any 7-char block
        for i in 0..=(cleaned.len() - 7) {
            let candidate = &cleaned[i..i + 7];
            let corrected: String = candidate
                .chars()
                .enumerate()
                .map(|(j, c)| match j {
                    0 | 1 | 5 | 6 => match c {
                        '0' => 'O',
                        '1' => 'I',
                        '2' => 'Z',
                        '5' => 'S',
                        '6' => 'G',
                        '8' => 'B',
                        _ => c,
                    },
                    2..=4 => match c {
                        'O' => '0',
                        'I' => '1',
                        'Z' => '2',
                        'S' => '5',
                        'G' => '6',
                        'B' => '8',
                        _ => c,
                    },
                    _ => c,
                })
                .collect();

            if PLATE_REGEX.is_match(&corrected) {
                return Some(corrected);
            }
        }
    }

    // Fallback: just return the cleaned string if it's potentially a plate
    if cleaned.len() >= 4 {
        return Some(cleaned);
    }

    None
}

/// Uppercase, strip non-alphanumeric chars.
pub fn normalise_plate(raw: &str) -> String {
    raw.to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────────
