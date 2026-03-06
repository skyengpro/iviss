use crate::dto::users::UserRole;
use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const SHIFT_TOKEN_TTL: Duration = Duration::from_secs(8 * 3600);
const REFRESH_TOKEN_TTL: Duration = Duration::from_secs(30 * 24 * 3600);
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,
    pub device_id: Uuid,
    pub role: String,
    pub exp: usize,
    pub jti: Uuid,
    pub shift_expires_at: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub device_id: Uuid,
    pub exp: usize,
    pub jti: Uuid,
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_jti: Uuid, // stored hashed in DB
    pub shift_expires_at: usize,
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

    pub fn issue_access_token(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        role: UserRole,
    ) -> Result<String> {
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
            shift_expires_at: exp,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        encode(&header, &claims, &self.encoding_key).context("Failed to sign access token")
    }

    /// Issues a shift access token (8h) — used for daily OTP login
    pub fn issue_shift_token(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        role: UserRole,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time before UNIX_EPOCH")?
            .as_secs();

        let exp = now
            .saturating_add(SHIFT_TOKEN_TTL.as_secs())
            .try_into()
            .unwrap_or(0usize);

        let claims = AccessTokenClaims {
            sub: user_id,
            device_id,
            role: role.as_str().to_string(),
            exp,
            jti: Uuid::new_v4(),
            shift_expires_at: exp, // same as exp — semantic claim for frontend
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        encode(&header, &claims, &self.encoding_key).context("Failed to sign shift token")
    }

    /// Issues a refresh token (30 days) — stored hashed in DB
    pub fn issue_refresh_token(&self, user_id: Uuid, device_id: Uuid) -> Result<(String, Uuid)> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time before UNIX_EPOCH")?
            .as_secs();

        let exp = now
            .saturating_add(REFRESH_TOKEN_TTL.as_secs())
            .try_into()
            .unwrap_or(0usize);

        let jti = Uuid::new_v4();

        let claims = RefreshTokenClaims {
            sub: user_id,
            device_id,
            exp,
            jti,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        let token =
            encode(&header, &claims, &self.encoding_key).context("Failed to sign refresh token")?;

        Ok((token, jti))
    }

    /// Issues a shift token + refresh token pair — used for daily OTP login
    pub fn issue_shift_token_pair(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        role: UserRole,
    ) -> Result<TokenPair> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time before UNIX_EPOCH")?
            .as_secs();

        let shift_exp = now
            .saturating_add(SHIFT_TOKEN_TTL.as_secs())
            .try_into()
            .unwrap_or(0usize);

        let access_claims = AccessTokenClaims {
            sub: user_id,
            device_id,
            role: role.as_str().to_string(),
            exp: shift_exp,
            jti: Uuid::new_v4(),
            shift_expires_at: shift_exp,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        let access_token = encode(&header, &access_claims, &self.encoding_key)
            .context("Failed to sign shift access token")?;

        let (refresh_token, refresh_jti) = self.issue_refresh_token(user_id, device_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            refresh_jti,
            shift_expires_at: shift_exp,
        })
    }
}
