use serde::Serialize;
use utoipa::ToSchema;

/// Successful OCR scan result data.
#[derive(Debug, Clone, Serialize, ToSchema, Default)]
pub struct ScanResultData {
    /// Normalized plate text (uppercase, no spaces). Empty if nothing detected.
    pub plate: String,
    /// Raw text as returned by the OCR engine before normalization.
    pub raw_text: String,
    /// Tesseract confidence score (0.0 – 1.0).
    pub confidence: f32,
    /// Whether the normalized plate matches the Cameroon format (XX###XX).
    pub format_valid: bool,
}

/// Error detail returned inside the scan response envelope.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScanErrorData {
    /// Machine-readable error code, e.g. `INVALID_IMAGE`.
    pub code: String,
    /// Human-readable error description.
    pub message: String,
}

/// Envelope response for `POST /api/v1/scan/plate`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScanPlateResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ScanResultData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ScanErrorData>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, ToSchema)]
pub struct ImageUploadRequest {
    #[schema(value_type = String, format = Binary)]
    pub image: String,
}
