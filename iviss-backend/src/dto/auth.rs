use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshNonceRequest {
    pub refresh_token: String,
    pub device_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshNonceResponse {
    pub challenge_id: Uuid,
    pub nonce: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshVerifyRequest {
    pub refresh_token: String,
    pub device_id: Uuid,
    pub challenge_id: Uuid,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshVerifyResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub device_id: Uuid,
    pub jti: Uuid,
    pub exp: i64,
    pub iat: i64,
}
