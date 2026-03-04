use crate::dto::users::UserRole;
use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,
    pub device_id: Uuid,
    pub role: String,
    pub exp: usize,
    pub jti: Uuid,
}

pub struct JwtService {
    encoding_key: EncodingKey,
}

impl JwtService {
    pub fn new(jwt_private_key_pem: &str) -> Result<Self> {
        let encoding_key = EncodingKey::from_rsa_pem(jwt_private_key_pem.as_bytes())
            .context("Failed to parse JWT RSA private key PEM")?;
        Ok(Self { encoding_key })
    }

    pub fn issue_access_token(&self, user_id: Uuid, device_id: Uuid, role: UserRole) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time before UNIX_EPOCH")?
            .as_secs();

        let exp = now
            .saturating_add(ACCESS_TOKEN_TTL.as_secs())
            .try_into()
            .unwrap_or(0usize);

        let claims = AccessTokenClaims {
            sub: user_id,
            device_id,
            role: role.as_str().to_string(),
            exp,
            jti: Uuid::new_v4(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        encode(&header, &claims, &self.encoding_key).context("Failed to sign access token")
    }
}
