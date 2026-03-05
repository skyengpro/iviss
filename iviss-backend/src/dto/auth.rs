
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
pub struct VerifyDailyLoginRequest {
    pub phone_number: String,
    pub device_id: uuid::Uuid,
    pub otp: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DailyLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub shift_expires_at: usize,
}
