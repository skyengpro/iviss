use image::codecs::bmp::BmpEncoder;
use image::{DynamicImage, ExtendedColorType, GenericImageView, GrayImage, Luma};
use leptess::{LepTess, Variable};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::dto::scan::ScanResultData;
use crate::errors::AppError;
use crate::services::ocr::timings::{OcrBudget, Stage, StageTimings};
use crate::utils::plate_format;

/// Sauvola window radius, derived from image height: a fixed radius does not
/// mean the same thing at 300px and at 1080px of input.
const SAUVOLA_RADIUS_DIVISOR: u32 = 8;
const SAUVOLA_RADIUS_MIN: u32 = 15;
const SAUVOLA_RADIUS_MAX: u32 = 100;

/// Sauvola sensitivity `k` (Leptonica documents 0.2–0.5, typically 0.35).
const SAUVOLA_K: f64 = 0.35;

/// Dynamic range of the local standard deviation in Sauvola's formula.
const SAUVOLA_STD_DEV_RANGE: f64 = 128.0;

/// White border added around the binarized crop. Tesseract reads characters
/// touching the edge poorly; *ImproveQuality* warns against larger borders.
const OCR_BORDER_PX: u32 = 30;

/// Fraction trimmed from each side before measuring polarity, so the surrounding
/// bodywork does not decide the polarity of the plate.
const POLARITY_INSET: f32 = 0.20;

/// Split point used to re-binarize after bilinear rotation.
const BILEVEL_MIDPOINT: u8 = 128;

/// Skew search range, in whole degrees, either side of zero.
const SKEW_SEARCH_DEGREES: i32 = 7;

/// Below this angle (radians) rotating costs more than it corrects.
const MIN_SKEW_RADIANS: f32 = 0.01;

/// Tesseract is told the source is 300 dpi; the crops are not scanned pages and
/// carry no resolution metadata of their own.
const SOURCE_RESOLUTION_DPI: i32 = 300;

/// Character set the plate formats are built from. Kept as a hint only: the LSTM
/// engine does not honour it reliably, so the real correction lives in
/// `plate_format`.
const PLATE_CHAR_WHITELIST: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Default tessdata location inside the Debian-based runtime image.
const DEFAULT_TESSDATA_PREFIX: &str = "/usr/share/tesseract-ocr/5/tessdata";

/// In-process deadline, checked between stages.
pub const OCR_STAGE_BUDGET: Duration = Duration::from_millis(8_500);

/// Handler-side deadline, covering queueing plus work. Deliberately longer than
/// [`OCR_STAGE_BUDGET`] so the pipeline gives up first and reports why.
pub const OCR_REQUEST_TIMEOUT: Duration = Duration::from_millis(9_000);

thread_local! {
    static TESSERACT: RefCell<Option<LepTess>> = const { RefCell::new(None) };
}

/// Caps concurrent OCR work at the number of cores. Without it a burst spreads
/// across Tokio's 512-thread blocking pool and every request degrades the next.
static OCR_PERMITS: Lazy<Arc<Semaphore>> = Lazy::new(|| {
    let workers = ocr_worker_count();
    tracing::info!("OCR concurrency limited to {workers} in-flight request(s)");
    Arc::new(Semaphore::new(workers))
});

fn ocr_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Acquire the right to run one OCR pipeline. Hold the permit for the whole
/// blocking task; dropping it releases the slot.
pub async fn acquire_ocr_permit() -> Result<OwnedSemaphorePermit, AppError> {
    OCR_PERMITS
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| AppError::internal_error(format!("OCR semaphore closed: {e}")))
}

/// Initialize one Tesseract instance per blocking worker at startup, so the
/// first real request does not pay for it.
pub fn warm_up_tesseract_pool() {
    let workers = ocr_worker_count();
    let barrier = Arc::new(Barrier::new(workers));

    for _ in 0..workers {
        let barrier = Arc::clone(&barrier);
        tokio::task::spawn_blocking(move || {
            match TesseractGuard::new() {
                Ok(guard) => drop(guard),
                Err(e) => tracing::warn!("Tesseract warm-up failed on this worker: {e}"),
            }
            barrier.wait();
        });
    }

    tracing::info!("Tesseract warm-up dispatched to {workers} worker(s)");
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
    let started = Instant::now();
    let budget = OcrBudget::new(OCR_STAGE_BUDGET);
    let mut timings = StageTimings::default();

    let img = decode_image(image_bytes, &mut timings)?;
    let result = scan_plate_image(&img, &mut timings, &budget);

    timings.total = started.elapsed();
    timings.emit("scan");

    finalize(result?, &timings)
}

/// Decode raw bytes, charging the cost to the decode stage.
pub fn decode_image(
    image_bytes: &[u8],
    timings: &mut StageTimings,
) -> Result<DynamicImage, AppError> {
    let img = timings
        .time(Stage::Decode, || image::load_from_memory(image_bytes))
        .map_err(|e| AppError::bad_request(format!("Cannot decode image: {e}")))?;

    let (width, height) = img.dimensions();
    timings.set_input_dimensions(width, height);
    Ok(img)
}

/// Run preprocessing and the OCR pass ensemble on an already-decoded image.
pub fn scan_plate_image(
    img: &DynamicImage,
    timings: &mut StageTimings,
    budget: &OcrBudget,
) -> Result<ScanResultData, AppError> {
    let gray = img.to_luma8();
    let prepared = preprocess(&gray, timings, budget)?;

    budget.check(Stage::TessInit)?;
    let mut tess = timings.time(Stage::TessInit, TesseractGuard::new)?;

    let page = encode_bmp(&prepared)?;
    run_pass_ensemble(&mut tess, &page, timings, budget)
}

/// Single exit point for a scan result.
pub fn finalize(res: ScanResultData, timings: &StageTimings) -> Result<ScanResultData, AppError> {
    tracing::info!(
        plate = %res.plate,
        plate_type = ?res.plate_type,
        format_valid = res.format_valid,
        confidence = res.confidence,
        passes = timings.passes,
        total_ms = timings.total.as_millis(),
        "OCR completed"
    );

    Ok(res)
}

// ── preprocessing pipeline ────────────────────────────────────────────────────

/// Contrast stretch → Sauvola → polarity normalisation → deskew → opening →
/// border.
///
/// Order matters, and this is not the order it used to be in:
///
/// * The skew search runs on a **binarized, polarity-normalised** image.
/// * The morphological opening runs **after** polarity normalisation: an
///   opening removes *light* structures, so on a light-on-dark plate it would
///   otherwise eat the glyphs instead of the noise.
/// * The border is added **once**, at the very end, on an image whose
///   background is already 255 — no second inverted image is produced, so the
///   dark frame that used to reach Tesseract on inverted passes cannot exist.
fn preprocess(
    gray: &GrayImage,
    timings: &mut StageTimings,
    budget: &OcrBudget,
) -> Result<GrayImage, AppError> {
    let radius =
        (gray.height() / SAUVOLA_RADIUS_DIVISOR).clamp(SAUVOLA_RADIUS_MIN, SAUVOLA_RADIUS_MAX);

    budget.check(Stage::Contrast)?;
    let stretched = timings.time(Stage::Contrast, || contrast_stretch_percentile(gray));

    budget.check(Stage::Threshold)?;
    let binary = timings.time(Stage::Threshold, || {
        let binarized = sauvola_threshold(&stretched, radius, SAUVOLA_K);
        if is_light_on_dark(&binarized) {
            invert_image(&binarized)
        } else {
            binarized
        }
    });

    budget.check(Stage::Deskew)?;
    let deskewed = timings.time(Stage::Deskew, || deskew(&binary));

    budget.check(Stage::Morphology)?;
    let opened = timings.time(Stage::Morphology, || morphology_open(&deskewed));

    budget.check(Stage::Border)?;
    Ok(timings.time(Stage::Border, || add_border(&opened, OCR_BORDER_PX, 255)))
}

/// Encode a grayscale image as an in-memory BMP for `set_image_from_mem`.
///
/// BMP is uncompressed, so this costs about a memcpy and loses nothing —
/// unlike the previous PNG round-trip through `/tmp`, which also leaked files
/// on every early return.
fn encode_bmp(img: &GrayImage) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    BmpEncoder::new(&mut buf)
        .encode(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::L8,
        )
        .map_err(|e| AppError::internal_error(format!("Failed to encode image for OCR: {e}")))?;
    Ok(buf)
}

// ── OCR passes ────────────────────────────────────────────────────────────────

/// Run the PSM ensemble and select a result.
///
/// PSM 7 and PSM 13 always both run: a single pass returning something
/// well-formed proves nothing, because the post-processing in `plate_format`
/// can turn noise into something well-formed. Agreement between two independent
/// segmentations is the actual evidence.
fn run_pass_ensemble(
    tess: &mut TesseractGuard,
    page: &[u8],
    timings: &mut StageTimings,
    budget: &OcrBudget,
) -> Result<ScanResultData, AppError> {
    let passes_started = Instant::now();

    budget.check(Stage::OcrPasses)?;
    set_page_seg_mode(tess, "7")?;
    let single_line = try_ocr(tess, page, "psm7");
    timings.count_pass();

    budget.check(Stage::OcrPasses)?;
    set_page_seg_mode(tess, "13")?;
    let raw_line = try_ocr(tess, page, "psm13");
    timings.count_pass();

    let selected = match (single_line, raw_line) {
        (Some(a), Some(b)) if passes_agree(&a, &b) => {
            // Report the stronger of the two measurements, not an average and
            // not an agreement bonus.
            let confidence = a.confidence.max(b.confidence);
            let mut agreed = if a.confidence >= b.confidence { a } else { b };
            agreed.confidence = confidence;
            agreed
        }
        (None, None) => {
            budget.check(Stage::OcrPasses)?;
            set_page_seg_mode(tess, "11")?;
            let sparse = try_ocr(tess, page, "psm11");
            timings.count_pass();
            pick_best_ensemble(vec![sparse])
        }
        (a, b) => pick_best_ensemble(vec![a, b]),
    };

    timings.record(Stage::OcrPasses, passes_started.elapsed());
    Ok(selected)
}

/// Two passes agree when both produced the same well-formed plate.
fn passes_agree(a: &ScanResultData, b: &ScanResultData) -> bool {
    a.format_valid && b.format_valid && !a.plate.is_empty() && a.plate == b.plate
}

fn set_page_seg_mode(tess: &mut LepTess, mode: &str) -> Result<(), AppError> {
    tess.set_variable(Variable::TesseditPagesegMode, mode)
        .map_err(|e| AppError::internal_error(format!("Failed to set PSM {mode}: {e}")))
}

pub fn take_tesseract() -> Result<LepTess, AppError> {
    TESSERACT.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            let tessdata = tessdata_prefix();
            let mut tess = LepTess::new(Some(&tessdata), "eng")
                .map_err(|e| AppError::internal_error(format!("Failed to init Tesseract: {e}")))?;
            configure_tesseract(&mut tess)?;
            *slot = Some(tess);
        }

        slot.take()
            .ok_or_else(|| AppError::internal_error("Tesseract slot was not initialized"))
    })
}

pub fn put_tesseract(tess: LepTess) {
    TESSERACT.with(|cell| {
        *cell.borrow_mut() = Some(tess);
    });
}

/// Read from the environment first so the binary stays runnable outside the
/// Docker image.
fn tessdata_prefix() -> String {
    std::env::var("TESSDATA_PREFIX").unwrap_or_else(|_| DEFAULT_TESSDATA_PREFIX.to_string())
}

/// Engine settings applied once, at instance creation.
///
/// The OCR engine mode cannot be set here: `leptess 0.14` exposes no OEM
/// parameter on `LepTess::new`, `tessedit_ocr_engine_mode` is read at Init, and
/// the underlying `TessApi` is private. The default is already LSTM on `eng`.
fn configure_tesseract(tess: &mut LepTess) -> Result<(), AppError> {
    let settings = [
        // A language model over an alphanumeric code bends plates like `CE128BC`
        // towards English vocabulary.
        (Variable::LoadSystemDawg, "0"),
        (Variable::LoadFreqDawg, "0"),
        // Polarity is normalised in `preprocess`; letting Tesseract retry the
        // inverted image silently doubles the compute.
        (Variable::TesseditDoInvert, "0"),
        (Variable::TesseditCharWhitelist, PLATE_CHAR_WHITELIST),
    ];

    for (variable, value) in settings {
        tess.set_variable(variable, value).map_err(|e| {
            AppError::internal_error(format!(
                "Failed to set Tesseract variable {variable:?}: {e}"
            ))
        })?;
    }

    Ok(())
}

/// Attempt OCR on an in-memory image. Returns `None` when the pass read no text.
pub fn try_ocr(tess: &mut LepTess, image: &[u8], label: &str) -> Option<ScanResultData> {
    tess.set_image_from_mem(image).ok()?;
    tess.set_source_resolution(SOURCE_RESOLUTION_DPI);

    let raw_text = tess.get_utf8_text().unwrap_or_default();
    let trimmed = raw_text.trim();
    let confidence = tess.mean_text_conf() as f32 / 100.0;
    let extracted = extract_plate_fuzzy(trimmed);
    let plate_match = extracted.as_deref().and_then(plate_format::classify);
    let format_valid = plate_match.is_some();
    let plate_type = plate_match.map(|m| m.category.as_str().to_string());

    tracing::debug!(
        pass = label,
        raw = trimmed,
        confidence,
        extracted = ?extracted,
        format_valid,
        "OCR pass"
    );

    if trimmed.is_empty() {
        return None;
    }

    Some(ScanResultData {
        plate: extracted.unwrap_or_default(),
        raw_text: trimmed.to_string(),
        confidence,
        format_valid,
        plate_type,
    })
}

/// Pick the best result from an ensemble of candidates.
pub fn pick_best_ensemble(candidates: Vec<Option<ScanResultData>>) -> ScanResultData {
    let (with_plate, without_plate): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .flatten()
        .partition(|cand| !cand.plate.is_empty());

    let pool = if with_plate.is_empty() {
        without_plate
    } else {
        with_plate
    };

    pool.into_iter()
        .reduce(|best, cand| {
            if (cand.format_valid, cand.confidence) > (best.format_valid, best.confidence) {
                cand
            } else {
                best
            }
        })
        .unwrap_or_default()
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

/// Sauvola adaptive binarization: `t = m · (1 − k · (1 − s / 128))`.
///
/// Two integral images (sums and sums of squares) keep the cost at O(1) per
/// pixel, exactly as the local-mean version. Window statistics use `f64`: on a
/// 100px radius the squared sums reach ~2.6e9, well past `f32` precision.
pub fn sauvola_threshold(img: &GrayImage, radius: u32, k: f64) -> GrayImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w == 0 || h == 0 {
        return img.clone();
    }

    let iw = w + 1;
    let pixels = img.as_raw();

    let mut sums = vec![0i64; iw * (h + 1)];
    let mut squares = vec![0i64; iw * (h + 1)];

    for y in 0..h {
        let mut row_sum = 0i64;
        let mut row_square = 0i64;
        let row_off = y * w;
        let curr_row = (y + 1) * iw;
        let prev_row = y * iw;

        for x in 0..w {
            let value = pixels[row_off + x] as i64;
            row_sum += value;
            row_square += value * value;
            sums[curr_row + x + 1] = row_sum + sums[prev_row + x + 1];
            squares[curr_row + x + 1] = row_square + squares[prev_row + x + 1];
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

            let count = ((x2 - x1) * (y2 - y1)) as f64;
            let window = |acc: &[i64]| {
                (acc[y2 * iw + x2] - acc[y1 * iw + x2] - acc[y2 * iw + x1] + acc[y1 * iw + x1])
                    as f64
            };

            let mean = window(&sums) / count;
            let variance = (window(&squares) / count - mean * mean).max(0.0);
            let std_dev = variance.sqrt();
            let threshold = mean * (1.0 - k * (1.0 - std_dev / SAUVOLA_STD_DEV_RANGE));

            out_pixels[row_off + x] = if f64::from(pixels[row_off + x]) > threshold {
                255
            } else {
                0
            };
        }
    }

    out
}

/// Whether a binarized image carries light glyphs on a dark background.
///
/// Measured on the central region only (a 20% inset per side). Across the whole
/// frame the margin is 5–10 points against a 50% tipping point, and with
/// `tessedit_do_invert=0` a wrong call is unrecoverable.
pub fn is_light_on_dark(binary: &GrayImage) -> bool {
    let (w, h) = binary.dimensions();
    if w == 0 || h == 0 {
        return false;
    }

    let inset_x = (w as f32 * POLARITY_INSET) as u32;
    let inset_y = (h as f32 * POLARITY_INSET) as u32;
    let x_end = w - inset_x;
    let y_end = h - inset_y;

    let raw = binary.as_raw();
    let mut dark = 0u64;

    for y in inset_y..y_end {
        let row_off = (y * w) as usize;
        for x in inset_x..x_end {
            if raw[row_off + x as usize] < BILEVEL_MIDPOINT {
                dark += 1;
            }
        }
    }

    let total = u64::from(x_end - inset_x) * u64::from(y_end - inset_y);
    total > 0 && dark * 2 > total
}

/// Find the rotation (radians) that best aligns text rows, by maximising the
/// variance of the horizontal projection profile.
///
/// The input must already be binarized **and** polarity-normalised: on
/// greyscale the variance is driven by broad luminance areas rather than by
/// text rows, and the chosen angle varies frame to frame on a plate that is
/// perfectly straight.
pub fn estimate_skew_angle(binary: &GrayImage) -> f32 {
    use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

    let (w, h) = binary.dimensions();
    if w == 0 || h == 0 {
        return 0.0;
    }

    // Seed with 0°, so an ambiguous image is left unrotated rather than
    // rotated by whichever angle was evaluated first.
    let mut best_angle = 0.0f32;
    let mut max_variance = projection_variance(binary);

    for angle_deg in -SKEW_SEARCH_DEGREES..=SKEW_SEARCH_DEGREES {
        if angle_deg == 0 {
            continue;
        }

        let rad = (angle_deg as f32).to_radians();
        let rotated = rotate_about_center(binary, rad, Interpolation::Bilinear, Luma([255]));
        let variance = projection_variance(&rotated);

        if variance > max_variance {
            max_variance = variance;
            best_angle = rad;
        }
    }

    best_angle
}

fn projection_variance(img: &GrayImage) -> f64 {
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w == 0 || h == 0 {
        return 0.0;
    }

    let raw = img.as_raw();
    let row_sums: Vec<f64> = (0..h)
        .map(|y| {
            raw[y * w..(y + 1) * w]
                .iter()
                .map(|&px| f64::from(px))
                .sum::<f64>()
        })
        .collect();

    let mean = row_sums.iter().sum::<f64>() / h as f64;
    row_sums
        .iter()
        .map(|sum| {
            let diff = sum - mean;
            diff * diff
        })
        .sum::<f64>()
        / h as f64
}

/// Rotation correction on a binarized, polarity-normalised image.
pub fn deskew(binary: &GrayImage) -> GrayImage {
    use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

    let angle = estimate_skew_angle(binary);
    if angle.abs() <= MIN_SKEW_RADIANS {
        return binary.clone();
    }

    let rotated = rotate_about_center(binary, angle, Interpolation::Bilinear, Luma([255]));
    to_bilevel(rotated)
}

fn to_bilevel(mut img: GrayImage) -> GrayImage {
    for px in img.iter_mut() {
        *px = if *px >= BILEVEL_MIDPOINT { 255 } else { 0 };
    }
    img
}

/// 3x3 morphological opening (erosion then dilation).
pub fn morphology_open(img: &GrayImage) -> GrayImage {
    let (w, h) = img.dimensions();
    if w < 3 || h < 3 {
        return img.clone();
    }

    let eroded = separable_3x3(img, |a, b, c| a.min(b).min(c));
    separable_3x3(&eroded, |a, b, c| a.max(b).max(c))
}

/// Apply a separable 3x3 rank filter: horizontally over every row, then
/// vertically over the interior rows.
fn separable_3x3(img: &GrayImage, combine: fn(u8, u8, u8) -> u8) -> GrayImage {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let src = img.as_raw();

    let mut horizontal = src.clone();
    for y in 0..h {
        let row_off = y * w;
        for x in 1..(w - 1) {
            horizontal[row_off + x] =
                combine(src[row_off + x - 1], src[row_off + x], src[row_off + x + 1]);
        }
    }

    let mut out = src.clone();
    for y in 1..(h - 1) {
        let row_off = y * w;
        for x in 1..(w - 1) {
            out[row_off + x] = combine(
                horizontal[row_off - w + x],
                horizontal[row_off + x],
                horizontal[row_off + w + x],
            );
        }
    }

    GrayImage::from_raw(img.width(), img.height(), out).unwrap_or_else(|| img.clone())
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
    let mut out = GrayImage::from_pixel(w + 2 * border, h + 2 * border, Luma([color]));
    image::imageops::replace(&mut out, img, border as i64, border as i64);
    out
}

/// Extract and correct a Cameroon plate candidate from noisy OCR text.
pub fn extract_plate_fuzzy(raw: &str) -> Option<String> {
    plate_format::fuzzy_correct(raw).map(|found| found.plate)
}

/// Uppercase, strip non-alphanumeric chars.
pub fn normalise_plate(raw: &str) -> String {
    plate_format::normalise(raw)
}
