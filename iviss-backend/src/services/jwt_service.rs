use crate::dto::users::UserRole;
use anyhow::{anyhow, Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding};
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(5 * 60);
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
        // 1. Aggressive cleanup of the input string
        let mut raw = jwt_private_key_pem.trim().to_string();
        
        // Remove outer quotes if they exist (common in shell-passed env vars)
        if (raw.starts_with('"') && raw.ends_with('"')) || (raw.starts_with('\'') && raw.ends_with('\'')) {
            raw = raw[1..raw.len()-1].trim().to_string();
        }
        
        // Handle literal \n and \r sequences from environment variables
        let normalized = raw.replace("\\n", "\n").replace("\\\\n", "\n").replace("\\r", "");
        
        // 2. Try standard jsonwebtoken parsing first
        match EncodingKey::from_rsa_pem(normalized.as_bytes()) {
            Ok(key) => Ok(Self { encoding_key: key }),
            Err(e) => {
                tracing::warn!(error = %e, "Initial JWT PEM parsing failed, attempting aggressive recovery...");
                
                // 3. Extract base64 content only (removes headers, footers, and all whitespace)
                let inner_base64 = normalized
                    .lines()
                    .filter(|l| !l.trim().starts_with("---"))
                    .collect::<String>()
                    .chars()
                    .filter(|c| !c.is_whitespace() && c != &'\"' && c != &'\'')
                    .collect::<String>();
                
                if inner_base64.len() < 100 {
                    return Err(anyhow!("JWT key content is too short or empty. Ensure JWT_PRIVATE_KEY_PEM is set correctly."));
                }

                // Try to parse using the rsa crate directly from DER (bypasses all PEM formatting issues)
                let der = base64::Engine::decode(
                    &base64::prelude::BASE64_STANDARD, 
                    &inner_base64.replace("-", "+").replace("_", "/")
                ).map_err(|err| anyhow!("Failed to base64-decode JWT key content: {}", err))?;
                
                let rsa_key = RsaPrivateKey::from_pkcs8_der(&der)
                    .or_else(|_| RsaPrivateKey::from_pkcs1_der(&der))
                    .map_err(|err| anyhow!("All RSA DER parsing attempts (PKCS#8 and PKCS#1) failed: {}. Ensure your key is a valid RSA private key.", err))?;

                // Export back to a clean PEM for jsonwebtoken compatibility
                let clean_pem = rsa_key.to_pkcs8_pem(LineEnding::LF)
                    .map_err(|_| anyhow!("Failed to re-normalize valid RSA key back to PEM"))?;
                
                let encoding_key = EncodingKey::from_rsa_pem(clean_pem.as_bytes())
                    .context("Failed to parse the final re-normalized JWT RSA private key")?;
                
                Ok(Self { encoding_key })
            }
        }
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
