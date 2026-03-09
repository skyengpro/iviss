use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Json};

use crate::dto::scan::{ScanErrorData, ScanPlateResponse, ScanResultData};
use crate::services::photo_ocr_service;

/// Maximum allowed image size: 8 MB (photo captures tend to be larger).
const MAX_IMAGE_SIZE: usize = 8 * 1024 * 1024;

/// Allowed MIME types for the uploaded image.
const ALLOWED_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png"];

/// Hard OCR timeout budget (server-side).
/// This keeps the photo endpoint responsive even if OCR gets slow.
const OCR_TIMEOUT_MS: u64 = 9000;

// ── POST /api/v1/photo/plate ─────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/photo/plate",
    tag = "photo",
    operation_id = "photoPlate",
    responses(
        (status = 200, description = "OCR result (photo capture)",      body = ScanPlateResponse),
        (status = 400, description = "Invalid or missing image",         body = ScanPlateResponse),
        (status = 415, description = "Unsupported media type",           body = ScanPlateResponse),
        (status = 500, description = "Internal OCR error",               body = ScanPlateResponse),
    ),
)]
pub async fn photo_plate(mut multipart: Multipart) -> impl IntoResponse {
    let (content_type, image_bytes) = match extract_image_field(&mut multipart).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if let Some(ref ct) = content_type {
        if !ALLOWED_CONTENT_TYPES.iter().any(|a| ct.starts_with(a)) {
            return error_response(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "Only JPEG and PNG images are accepted",
            );
        }
    }

    if image_bytes.len() > MAX_IMAGE_SIZE {
        return error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_IMAGE",
            "Image size exceeds 8 MB limit",
        );
    }

    // Photo is a single-shot high-res capture. We can afford a slightly heavier
    // pipeline than live scanning while still reusing the OCR engine.
    // Still, we keep a hard timeout so the endpoint remains responsive.
    let mut handle =
        tokio::task::spawn_blocking(move || photo_ocr_service::photo_plate(&image_bytes));
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(OCR_TIMEOUT_MS),
        &mut handle,
    )
    .await;

    match result {
        Ok(joined) => match joined {
            Ok(Ok(scan_data)) => success_response(scan_data),
            Ok(Err(app_err)) => {
                tracing::warn!("Photo OCR processing error: {app_err}");
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "OCR_ERROR",
                    "OCR processing failed",
                )
            }
            Err(join_err) => {
                tracing::error!("Photo OCR task panicked: {join_err}");
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "OCR_ERROR",
                    "Internal Server Error",
                )
            }
        },
        Err(_) => {
            handle.abort();
            error_response(
                StatusCode::GATEWAY_TIMEOUT,
                "OCR_TIMEOUT",
                "OCR processing timed out",
            )
        }
    }
}

// ── helpers (kept local to avoid altering scan handler behavior) ──────────────

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
