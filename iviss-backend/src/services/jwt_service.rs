use crate::errors::AppError;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

const ACCESS_TOKEN_DURATION_SECS: i64 = 8 * 3600; // 8h shift
const REFRESH_TOKEN_DURATION_SECS: i64 = 30 * 24 * 3600; // 30 days

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub jti: String,
    pub device_id: Uuid,
    pub shift_expires_at: usize, // business claim — same value as exp
}

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_token_jti: String, // used to store hashed in DB
    pub shift_expires_at: usize,
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Issues an access token (8h) + refresh token (30 days)
    pub fn issue_token_pair(&self, user_id: Uuid, device_id: Uuid) -> Result<TokenPair, AppError> {
        let (access_token, shift_expires_at) = self.generate_access_token(user_id, device_id)?;
        let (refresh_token, refresh_token_jti) = self.generate_refresh_token(user_id, device_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            refresh_token_jti,
            shift_expires_at,
        })
    }

    fn generate_access_token(
        &self,
        user_id: Uuid,
        device_id: Uuid,
    ) -> Result<(String, usize), AppError> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let exp = (now + ACCESS_TOKEN_DURATION_SECS) as usize;

        let claims = JwtClaims {
            sub: user_id,
            exp,
            jti: Uuid::new_v4().to_string(),
            device_id,
            shift_expires_at: exp, // same as exp — semantic claim for frontend
        };

        let token = self.sign(claims)?;
        Ok((token, exp))
    }

    fn generate_refresh_token(
        &self,
        user_id: Uuid,
        device_id: Uuid,
    ) -> Result<(String, String), AppError> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let exp = (now + REFRESH_TOKEN_DURATION_SECS) as usize;
        let jti = Uuid::new_v4().to_string();

        let claims = JwtClaims {
            sub: user_id,
            exp,
            jti: jti.clone(),
            device_id,
            shift_expires_at: (now + ACCESS_TOKEN_DURATION_SECS) as usize,
        };

        let token = self.sign(claims)?;
        Ok((token, jti))
    }

    fn sign(&self, claims: JwtClaims) -> Result<String, AppError> {
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT signing failed: {}", e)))
    }
}
