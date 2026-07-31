//! Per-stage instrumentation and deadline enforcement for the OCR pipeline.
//!
//! Two concerns live here:
//!
//! * [`StageTimings`] — where the time actually goes, published as structured
//!   logs and Prometheus histograms so a regression is visible per stage rather
//!   than as a single opaque total.
//! * [`OcrBudget`] — a wall-clock deadline checked between stages. A request the
//!   client has already given up on must stop burning CPU at the next
//!   checkpoint: `JoinHandle::abort()` is a no-op on a `spawn_blocking` task
//!   that has already started, so cooperative checks are the only lever.

use std::time::{Duration, Instant};

use crate::errors::AppError;

/// Pipeline stages, used both as metric labels and as budget checkpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Decode,
    Crop,
    Deskew,
    Contrast,
    Threshold,
    Morphology,
    Border,
    TessInit,
    OcrPasses,
}

impl Stage {
    pub const ALL: [Stage; 9] = [
        Stage::Decode,
        Stage::Crop,
        Stage::Deskew,
        Stage::Contrast,
        Stage::Threshold,
        Stage::Morphology,
        Stage::Border,
        Stage::TessInit,
        Stage::OcrPasses,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Decode => "decode",
            Stage::Crop => "crop",
            Stage::Deskew => "deskew",
            Stage::Contrast => "contrast",
            Stage::Threshold => "threshold",
            Stage::Morphology => "morphology",
            Stage::Border => "border",
            Stage::TessInit => "tess_init",
            Stage::OcrPasses => "ocr_passes",
        }
    }
}

/// Accumulated duration per stage for a single request.
///
/// Durations accumulate, so a path that runs the pipeline twice (the photo
/// fallback pass) reports the sum rather than the last run. `total` is set by
/// the entry point from its own `Instant`, never by summing stages — summing
/// would miss decode and crop and produce `preprocessing > total`.
#[derive(Debug, Clone, Default)]
pub struct StageTimings {
    durations: [Duration; Stage::ALL.len()],
    pub total: Duration,
    pub input_width: u32,
    pub input_height: u32,
    pub passes: u32,
}

impl StageTimings {
    pub fn record(&mut self, stage: Stage, elapsed: Duration) {
        self.durations[stage as usize] += elapsed;
    }

    /// Run `f`, charging its wall-clock duration to `stage`.
    pub fn time<T>(&mut self, stage: Stage, f: impl FnOnce() -> T) -> T {
        let started = Instant::now();
        let out = f();
        self.record(stage, started.elapsed());
        out
    }

    pub fn get(&self, stage: Stage) -> Duration {
        self.durations[stage as usize]
    }

    pub fn set_input_dimensions(&mut self, width: u32, height: u32) {
        self.input_width = width;
        self.input_height = height;
    }

    /// Count one Tesseract recognition pass.
    pub fn count_pass(&mut self) {
        self.passes += 1;
    }

    /// Publish the breakdown as Prometheus histograms plus one debug log line.
    ///
    /// `path` distinguishes the live scan pipeline from the photo pipeline.
    pub fn emit(&self, path: &'static str) {
        for stage in Stage::ALL {
            metrics::histogram!(
                "iviss_ocr_stage_duration_seconds",
                "stage" => stage.as_str(),
                "path" => path,
            )
            .record(self.get(stage).as_secs_f64());
        }

        metrics::histogram!("iviss_ocr_duration_seconds", "path" => path)
            .record(self.total.as_secs_f64());
        metrics::histogram!("iviss_ocr_passes", "path" => path).record(f64::from(self.passes));
        metrics::histogram!("iviss_ocr_input_pixels", "path" => path)
            .record(f64::from(self.input_width) * f64::from(self.input_height));

        tracing::debug!(
            path,
            input_width = self.input_width,
            input_height = self.input_height,
            passes = self.passes,
            decode_ms = self.get(Stage::Decode).as_millis(),
            crop_ms = self.get(Stage::Crop).as_millis(),
            contrast_ms = self.get(Stage::Contrast).as_millis(),
            threshold_ms = self.get(Stage::Threshold).as_millis(),
            deskew_ms = self.get(Stage::Deskew).as_millis(),
            morphology_ms = self.get(Stage::Morphology).as_millis(),
            border_ms = self.get(Stage::Border).as_millis(),
            tess_init_ms = self.get(Stage::TessInit).as_millis(),
            ocr_passes_ms = self.get(Stage::OcrPasses).as_millis(),
            total_ms = self.total.as_millis(),
            "OCR stage breakdown"
        );
    }
}

/// Raised when the pipeline runs past its deadline.
///
/// Carried inside `AppError::Internal` so handlers can map it to a gateway
/// timeout without adding a variant to the OpenAPI-exposed `ErrorCode` enum.
#[derive(Debug, thiserror::Error)]
#[error("OCR budget exceeded at stage {stage} after {elapsed:?}")]
pub struct OcrBudgetExceeded {
    pub stage: &'static str,
    pub elapsed: Duration,
}

/// Wall-clock deadline for one OCR request.
#[derive(Debug, Clone, Copy)]
pub struct OcrBudget {
    started: Instant,
    budget: Duration,
}

impl OcrBudget {
    pub fn new(budget: Duration) -> Self {
        Self {
            started: Instant::now(),
            budget,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn remaining(&self) -> Duration {
        self.budget.saturating_sub(self.elapsed())
    }

    pub fn is_exceeded(&self) -> bool {
        self.remaining().is_zero()
    }

    /// Abort the pipeline if the deadline has passed. Call between stages.
    pub fn check(&self, stage: Stage) -> Result<(), AppError> {
        if !self.is_exceeded() {
            return Ok(());
        }

        let elapsed = self.elapsed();
        metrics::counter!("iviss_ocr_budget_exceeded_total", "stage" => stage.as_str())
            .increment(1);
        tracing::warn!(
            stage = stage.as_str(),
            elapsed_ms = elapsed.as_millis(),
            budget_ms = self.budget.as_millis(),
            "OCR budget exhausted, abandoning request"
        );

        Err(AppError::Internal(anyhow::Error::new(OcrBudgetExceeded {
            stage: stage.as_str(),
            elapsed,
        })))
    }
}

/// Whether an error came from [`OcrBudget::check`] rather than a real failure.
pub fn is_budget_exceeded(err: &AppError) -> bool {
    match err {
        AppError::Internal(inner) => inner.downcast_ref::<OcrBudgetExceeded>().is_some(),
        _ => false,
    }
}
