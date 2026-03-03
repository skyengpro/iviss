use image::{GenericImageView, GrayImage};
use once_cell::sync::Lazy;
use regex::Regex;
use leptess::{LepTess, Variable};

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;

/// Cameroon plate format: 2 letters + 3 digits + 2 letters (e.g. CE128BC).
static PLATE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}[0-9]{3}[A-Z]{2}$").unwrap());

/// Target width (px) for the resized image fed to Tesseract.
const TARGET_WIDTH: u32 = 1200;

/// Radius (in pixels) for the adaptive threshold sliding window.
const ADAPTIVE_RADIUS: u32 = 40;

/// Offset subtracted from the local mean when applying adaptive threshold.
const ADAPTIVE_C: i16 = 5;

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
    tracing::info!("Received image for OCR: {}x{} ({} bytes), load took {:?}", width, height, image_bytes.len(), load_elapsed);

    // 2. Convert to 8-bit grayscale
    let gray = img.to_luma8();

    // 3. Preprocessing: contrast stretch → adaptive threshold
    let process_start = std::time::Instant::now();
    let stretched = contrast_stretch(&gray);
    let binary = adaptive_threshold(&stretched, ADAPTIVE_RADIUS, ADAPTIVE_C);
    let inverted = invert_image(&binary);
    
    // Add 30px white border — Tesseract works much better when chars aren't touching edges
    let binary = add_border(&binary, 30, 255);
    let inverted = add_border(&inverted, 30, 255);
    
    let process_elapsed = process_start.elapsed();

    // 4. Initialize Tesseract
    let mut tess = LepTess::new(Some("/usr/share/tesseract-ocr/5/tessdata"), "eng")
        .map_err(|e| AppError::internal_error(format!("Failed to init Tesseract: {e}")))?;

    tess.set_variable(Variable::TesseditCharWhitelist, "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")
        .map_err(|e| AppError::internal_error(format!("Failed to set whitelist: {e}")))?;

    let tesseract_start = std::time::Instant::now();
    
    // --- MODE 1: PSM 7 (Single Text Line) ---
    tess.set_variable(Variable::TesseditPagesegMode, "7")
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM 7: {e}")))?;
    let r_b7 = try_ocr(&mut tess, &binary, "binary-psm7");
    let r_i7 = try_ocr(&mut tess, &inverted, "inverted-psm7");

    // Check if we already found a winner
    if let Some(ref res) = r_b7 { if res.format_valid { return finalize(res.clone(), process_elapsed, tesseract_start.elapsed(), start_total.elapsed()); } }
    if let Some(ref res) = r_i7 { if res.format_valid { return finalize(res.clone(), process_elapsed, tesseract_start.elapsed(), start_total.elapsed()); } }

    // --- MODE 2: PSM 11 (Sparse Text) --- Fallback
    tess.set_variable(Variable::TesseditPagesegMode, "11")
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM 11: {e}")))?;
    let r_b11 = try_ocr(&mut tess, &binary, "binary-psm11");
    let r_i11 = try_ocr(&mut tess, &inverted, "inverted-psm11");

    // Save debug image (latest frame)
    let _ = binary.save("/tmp/ocr_debug_binary_latest.png");
    let _ = inverted.save("/tmp/ocr_debug_inverted_latest.png");

    let tesseract_elapsed = tesseract_start.elapsed();

    // 5. Result Selection
    let candidates = vec![r_b7, r_i7, r_b11, r_i11];
    let final_result = pick_best_ensemble(candidates);

    finalize(final_result, process_elapsed, tesseract_elapsed, start_total.elapsed())
}

/// Run OCR for a single-shot photo capture.
///
/// This is intentionally a separate entrypoint from `scan_plate` so the photo
/// mechanism can evolve independently (heavier preprocessing / different
/// candidate selection) without changing the live scanning behavior.
pub fn photo_plate(image_bytes: &[u8]) -> Result<ScanResultData, AppError> {
    // For now we reuse the same underlying pipeline as scan.
    // Separation is enforced at the API boundary + entrypoint level.
    scan_plate(image_bytes)
}

fn finalize(mut res: ScanResultData, proc: std::time::Duration, tess: std::time::Duration, total: std::time::Duration) -> Result<ScanResultData, AppError> {
    if res.format_valid {
        res.confidence = 0.90;
    } else if !res.plate.is_empty() {
        res.confidence = 0.50;
    }
    
    tracing::info!("Scan completed: process={:?}, tesseract={:?}, total={:?}, plate={:?} (conf={:.2})", 
        proc, tess, total, res.plate, res.confidence);

    Ok(res)
}

// ── OCR helper ────────────────────────────────────────────────────────────────

/// Attempt OCR on a single grayscale image using leptess.
fn try_ocr(tess: &mut LepTess, img: &GrayImage, label: &str) -> Option<ScanResultData> {
    // Write image to a temp file as PNG (bypasses potential BMP/memory issues in leptess)
    let tmp_path = format!("/tmp/ocr_tmp_{}.png", label.replace('-', "_"));
    img.save(&tmp_path).ok()?;

    // Load from file path (Leptonica handles PNG natively)
    tess.set_image(&tmp_path).ok()?;
    tess.set_source_resolution(300);

    let raw_text = tess.get_utf8_text().unwrap_or_default();
    let trimmed = raw_text.trim();
    let confidence = tess.mean_text_conf() as f32 / 100.0;
    let extracted = extract_plate_fuzzy(trimmed);
    let format_valid = extracted.as_ref().map(|p| PLATE_REGEX.is_match(p)).unwrap_or(false);

    tracing::info!("[{}] OCR raw: {:?} (conf: {:.2}), extracted: {:?}, valid: {}", 
        label, trimmed, confidence, extracted, format_valid);
    
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
fn pick_best_ensemble(candidates: Vec<Option<ScanResultData>>) -> ScanResultData {
    let mut best: Option<ScanResultData> = None;

    for cand in candidates.into_iter().flatten() {
        match &best {
            None => best = Some(cand),
            Some(curr) => {
                // Priority 1: Valid format
                if cand.format_valid && !curr.format_valid {
                    best = Some(cand);
                }
                // Priority 2: Higher confidence (if both valid or both invalid)
                else if (cand.format_valid == curr.format_valid) && (cand.confidence > curr.confidence) {
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

/// Min-max contrast stretch: maps the pixel range [min, max] → [0, 255].
fn contrast_stretch(img: &GrayImage) -> GrayImage {
    let pixels = img.as_raw();
    if pixels.is_empty() { return img.clone(); }

    let mut min_val = 255u8;
    let mut max_val = 0u8;
    
    for &px in pixels {
        if px < min_val { min_val = px; }
        if px > max_val { max_val = px; }
    }

    if max_val == min_val {
        return img.clone();
    }

    let range = (max_val - min_val) as f32;
    let (w, h) = img.dimensions();
    let mut out = GrayImage::new(w, h);
    
    // Predeterminlookup table for speed
    let mut lut = [0u8; 256];
    for i in 0..256 {
        lut[i] = (((i as f32 - min_val as f32).max(0.0) / range * 255.0) as u8).min(255);
    }

    for (out_px, &in_px) in out.iter_mut().zip(pixels.iter()) {
        *out_px = lut[in_px as usize];
    }
    
    out
}

/// Adaptive thresholding using a local mean (integral-image based, O(1) per pixel).
fn adaptive_threshold(img: &GrayImage, radius: u32, c: i16) -> GrayImage {
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
            let sum = integral[y2 * iw + x2] - integral[y1 * iw + x2]
                - integral[y2 * iw + x1] + integral[y1 * iw + x1];
            
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
fn invert_image(img: &GrayImage) -> GrayImage {
    let mut out = img.clone();
    for px in out.iter_mut() {
        *px = 255 - *px;
    }
    out
}

/// Add a solid color border around the image.
fn add_border(img: &GrayImage, border: u32, color: u8) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut out = GrayImage::from_pixel(w + 2 * border, h + 2 * border, image::Luma([color]));
    image::imageops::replace(&mut out, img, border as i64, border as i64);
    out
}


/// Cameroon plate: 2 letters + 3 digits + 2 letters.
/// Apply position-aware correction for common OCR misreads.
fn extract_plate_fuzzy(raw: &str) -> Option<String> {
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
            let candidate = &cleaned[i..i+7];
            let corrected: String = candidate.chars().enumerate().map(|(j, c)| {
                match j {
                    0 | 1 | 5 | 6 => match c {
                        '0' => 'O', '1' => 'I', '2' => 'Z', '5' => 'S', '6' => 'G', '8' => 'B', _ => c
                    },
                    2 | 3 | 4 => match c {
                        'O' => '0', 'I' => '1', 'Z' => '2', 'S' => '5', 'G' => '6', 'B' => '8', _ => c
                    },
                    _ => c,
                }
            }).collect();
            
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
    use image::Luma;

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
