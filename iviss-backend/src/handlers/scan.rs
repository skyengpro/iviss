use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Json};
use tracing::instrument;

#[allow(unused_imports)]
use crate::dto::scan::ImageUploadRequest;
use crate::dto::scan::{ScanErrorData, ScanPlateResponse, ScanResultData};
use crate::services::{ocr_service, ocr_timings};

/// Maximum allowed image size: 5 MB.
const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024;

/// Allowed MIME types for the uploaded image.
const ALLOWED_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png"];

// ── POST /api/v1/scan/plate ──────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/scan/plate",
    tag = "scanning",
    operation_id = "scanPlate",
    request_body(
        content = ImageUploadRequest,
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "OCR result (may have low confidence)", body = ScanPlateResponse),
        (status = 400, description = "Invalid or missing image",            body = ScanPlateResponse),
        (status = 415, description = "Unsupported media type",              body = ScanPlateResponse),
        (status = 500, description = "Internal OCR error",                  body = ScanPlateResponse),
    ),
)]
#[instrument(name = "scan.plate", skip(multipart))]
pub async fn scan_plate(mut multipart: Multipart) -> impl IntoResponse {
    // ── 1. Extract the `image` field from the multipart body ─────────────────
    let (content_type, image_bytes) = match extract_image_field(&mut multipart).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // ── 2. Validate content type ─────────────────────────────────────────────
    if let Some(ref ct) = content_type {
        if !ALLOWED_CONTENT_TYPES.iter().any(|a| ct.starts_with(a)) {
            return error_response(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "Only JPEG and PNG images are accepted",
            );
        }
    }
    // If no content_type header was sent with the field, we still try to
    // decode it and let the image crate reject it if necessary.

    // ── 3. Validate file size ────────────────────────────────────────────────
    if image_bytes.len() > MAX_IMAGE_SIZE {
        return error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_IMAGE",
            "Image size exceeds 5 MB limit",
        );
    }

    // ── 4. Run OCR pipeline ──────────────────────────────────────────────────
    // A single deadline covers both queueing and work. The permit caps
    // concurrent OCR at the core count: without it a burst spreads over Tokio's
    // blocking pool and every timeout degrades the next request.
    let deadline = tokio::time::Instant::now() + ocr_service::OCR_REQUEST_TIMEOUT;

    let permit = match tokio::time::timeout_at(deadline, ocr_service::acquire_ocr_permit()).await {
        Ok(Ok(permit)) => permit,
        Ok(Err(app_err)) => {
            tracing::error!("Failed to acquire OCR permit: {app_err}");
            metrics::counter!("iviss_scan_errors_total", "error_type" => "ocr_error").increment(1);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "OCR_ERROR",
                "OCR processing failed",
            );
        }
        Err(_) => {
            metrics::counter!("iviss_scan_errors_total", "error_type" => "timeout").increment(1);
            return error_response(
                StatusCode::GATEWAY_TIMEOUT,
                "OCR_TIMEOUT",
                "OCR processing timed out",
            );
        }
    };

    // Offload the CPU-heavy work to a blocking thread so we don't starve
    // the async Tokio runtime. The permit is released when the task ends.
    let plate_len = image_bytes.len();
    let mut handle = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        ocr_service::scan_plate(&image_bytes)
    });
    let result = tokio::time::timeout_at(deadline, &mut handle).await;

    match result {
        Ok(joined) => match joined {
            Ok(Ok(scan_data)) => {
                metrics::counter!("iviss_scans_total", "plate_length" => plate_len.to_string())
                    .increment(1);
                success_response(scan_data)
            }
            // The pipeline gives up on its own budget slightly before the
            // handler deadline, so this is a timeout, not an engine failure.
            Ok(Err(app_err)) if ocr_timings::is_budget_exceeded(&app_err) => {
                tracing::warn!("OCR abandoned: {app_err}");
                metrics::counter!("iviss_scan_errors_total", "error_type" => "timeout")
                    .increment(1);
                error_response(
                    StatusCode::GATEWAY_TIMEOUT,
                    "OCR_TIMEOUT",
                    "OCR processing timed out",
                )
            }
            Ok(Err(app_err)) => {
                tracing::warn!("OCR processing error: {app_err}");
                metrics::counter!("iviss_scan_errors_total", "error_type" => "ocr_error")
                    .increment(1);
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "OCR_ERROR",
                    "OCR processing failed",
                )
            }
            Err(join_err) => {
                tracing::error!("OCR task panicked: {join_err}");
                metrics::counter!("iviss_scan_errors_total", "error_type" => "panic").increment(1);
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "OCR_ERROR",
                    "Internal Server Error",
                )
            }
        },
        Err(_) => {
            // `abort()` cannot stop a `spawn_blocking` task that already
            // started; the in-pipeline budget check is what actually stops it.
            handle.abort();
            metrics::counter!("iviss_scan_errors_total", "error_type" => "timeout").increment(1);
            error_response(
                StatusCode::GATEWAY_TIMEOUT,
                "OCR_TIMEOUT",
                "OCR processing timed out",
            )
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Walk the multipart stream to find a field named `image` and return its
/// content-type and bytes. Returns an error response if the field is missing.
async fn extract_image_field(
    multipart: &mut Multipart,
) -> Result<(Option<String>, Vec<u8>), (StatusCode, Json<ScanPlateResponse>)> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name != "image" {
            continue;
        }

        let content_type = field.content_type().map(|s| s.to_string());
        let bytes = field.bytes().await.map_err(|e| {
            tracing::warn!("Failed to read multipart field: {e}");
            error_response_tuple(
                StatusCode::BAD_REQUEST,
                "INVALID_IMAGE",
                "Failed to read image data",
            )
        })?;

        return Ok((content_type, bytes.to_vec()));
    }

    Err(error_response_tuple(
        StatusCode::BAD_REQUEST,
        "INVALID_IMAGE",
        "Missing required field: image",
    ))
}

fn success_response(data: ScanResultData) -> (StatusCode, Json<ScanPlateResponse>) {
    (
        StatusCode::OK,
        Json(ScanPlateResponse {
            success: true,
            data: Some(data),
            error: None,
        }),
    )
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<ScanPlateResponse>) {
    error_response_tuple(status, code, message)
}

fn error_response_tuple(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<ScanPlateResponse>) {
    (
        status,
        Json(ScanPlateResponse {
            success: false,
            data: None,
            error: Some(ScanErrorData {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }),
    )
}
