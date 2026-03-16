use crate::dto::users::UserRole;
use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(3 * 60);
const SHIFT_TTL: Duration = Duration::from_secs(8 * 60 * 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,
    pub device_id: Uuid,
    pub role: String,
    pub shift_start: usize,
    pub shift_end: usize,
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

        let shift_start: usize = now.try_into().unwrap_or(0usize);
        let shift_end: usize = now
            .saturating_add(SHIFT_TTL.as_secs())
            .try_into()
            .unwrap_or(0usize);

        self.issue_access_token_with_shift(user_id, device_id, role, shift_start, shift_end)
    }

    pub fn issue_access_token_with_shift(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        role: UserRole,
        shift_start: usize,
        shift_end: usize,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System time before UNIX_EPOCH")?
            .as_secs();

        let exp: usize = now
            .saturating_add(ACCESS_TOKEN_TTL.as_secs())
            .try_into()
            .unwrap_or(0usize);

        let claims = AccessTokenClaims {
            sub: user_id,
            device_id,
            role: role.as_str().to_string(),
            shift_start,
            shift_end,
            exp,
            jti: Uuid::new_v4(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_string());

        encode(&header, &claims, &self.encoding_key).context("Failed to sign access token")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rsa::{
        pkcs8::{EncodePrivateKey, EncodePublicKey},
        RsaPrivateKey,
    };

    fn generate_test_keys() -> (String, String) {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate private key");
        let pub_key = priv_key.to_public_key();

        let priv_pem = priv_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .to_string();
        let pub_pem = pub_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();

        (priv_pem, pub_pem)
    }

    #[test]
    fn test_jwt_service_new_invalid_pem() {
        let result = JwtService::new("invalid pem");
        assert!(result.is_err());
    }

    #[test]
    fn test_issue_access_token() {
        let (priv_pem, pub_pem) = generate_test_keys();
        let svc = JwtService::new(&priv_pem).unwrap();

        let user_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let token = svc
            .issue_access_token(user_id, device_id, UserRole::Admin)
            .unwrap();

        // Verify token can be decoded
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(pub_pem.as_bytes()).unwrap();
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_exp = true;

        let decoded =
            jsonwebtoken::decode::<AccessTokenClaims>(&token, &decoding_key, &validation).unwrap();
        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.device_id, device_id);
        assert_eq!(decoded.claims.role, "admin");
    }

    #[test]
    fn test_issue_access_token_with_shift() {
        let (priv_pem, _pub_pem) = generate_test_keys();
        let svc = JwtService::new(&priv_pem).unwrap();

        let user_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let token = svc
            .issue_access_token_with_shift(user_id, device_id, UserRole::Manager, 1000, 2000)
            .unwrap();
        assert!(!token.is_empty());
    }
}
