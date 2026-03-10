use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendActivationResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RequestDailyLoginRequest {
    pub phone_number: String,
    pub device_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RequestDailyLoginResponse {
    pub message: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DailyLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub shift_expires_at: usize,
}

// ── Daily login DTOs ──────────────────────────────────────────────────────────


#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyDailyLoginRequest {
    pub badge_id: String,
    pub otp: String,
    pub device_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyDailyLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub shift_end: i64,
}