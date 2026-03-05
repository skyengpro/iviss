use crate::db::{DbPool, RedisPool};
use crate::dto::auth::AccessTokenClaims;
use crate::errors::AppError;
use anyhow::Context;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use deadpool_redis::redis::AsyncCommands;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

pub const NONCE_TTL_SECS: u64 = 90;
pub const ACCESS_TOKEN_TTL_SECS: u64 = 15 * 60;
const NONCE_KEY_PREFIX: &str = "refresh_nonce";

#[derive(Debug, Clone)]
pub struct NonceChallenge {
    pub challenge_id: Uuid,
    pub nonce: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshNonceEntry {
    token_hash: String,
    user_id: Uuid,
    device_id: Uuid,
    nonce: String,
}

#[derive(Debug, sqlx::FromRow)]
struct RefreshBindingRow {
    user_id: Uuid,
    device_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
struct RefreshBindingWithKeyRow {
    user_id: Uuid,
    device_id: Uuid,
    public_key: String,
}

pub struct RefreshService {
    db: DbPool,
    redis: RedisPool,
    jwt_secret: String,
}

impl RefreshService {
    pub fn new(db: DbPool, redis: RedisPool, jwt_secret: String) -> Self {
        Self {
            db,
            redis,
            jwt_secret,
        }
    }

    pub fn hash_refresh_token(refresh_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn create_nonce_challenge(
        &self,
        refresh_token: &str,
        device_id: Uuid,
    ) -> Result<NonceChallenge, AppError> {
        let token_hash = Self::hash_refresh_token(refresh_token);
        let binding = self
            .fetch_active_refresh_binding(&token_hash, device_id)
            .await?;

        let challenge_id = Uuid::new_v4();
        let nonce = Self::generate_nonce();
        let key = Self::redis_key(challenge_id);

        let entry = RefreshNonceEntry {
            token_hash,
            user_id: binding.user_id,
            device_id: binding.device_id,
            nonce: nonce.clone(),
        };

        let payload =
            serde_json::to_string(&entry).context("Failed to serialize refresh nonce entry")?;

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection for refresh nonce")?;

        conn.set_ex::<_, _, ()>(&key, payload, NONCE_TTL_SECS)
            .await
            .context("Failed to store refresh nonce challenge in Redis")?;

        Ok(NonceChallenge {
            challenge_id,
            nonce,
            expires_in_seconds: NONCE_TTL_SECS,
        })
    }

    pub async fn verify_and_issue_access_token(
        &self,
        refresh_token: &str,
        device_id: Uuid,
        challenge_id: Uuid,
        signature: &str,
    ) -> Result<String, AppError> {
        let token_hash = Self::hash_refresh_token(refresh_token);
        let nonce_entry = self.consume_nonce_challenge(challenge_id).await?;

        if nonce_entry.token_hash != token_hash || nonce_entry.device_id != device_id {
            self.revoke_refresh_token(&nonce_entry.token_hash).await?;
            return Err(AppError::unauthorized(
                "Refresh challenge does not match token or device",
            ));
        }

        let binding = self
            .fetch_active_refresh_binding_with_key(&token_hash, device_id)
            .await?;
        if self
            .verify_device_signature(&binding.public_key, &nonce_entry.nonce, signature)
            .is_err()
        {
            self.revoke_refresh_token(&token_hash).await?;
            return Err(AppError::unauthorized("Invalid device signature"));
        }

        self.issue_access_token(binding.user_id, binding.device_id)
    }

    async fn fetch_active_refresh_binding(
        &self,
        token_hash: &str,
        device_id: Uuid,
    ) -> Result<RefreshBindingRow, AppError> {
        sqlx::query_as::<_, RefreshBindingRow>(
            r#"
            SELECT rt.user_id, rt.device_id
            FROM refresh_tokens rt
            INNER JOIN users u ON u.id = rt.user_id
            INNER JOIN devices d ON d.id = rt.device_id
            WHERE rt.token_hash = $1
              AND rt.device_id = $2
              AND rt.revoked = FALSE
              AND rt.expires_at > NOW()
              AND u.deleted_at IS NULL
              AND u.status::TEXT = 'ACTIVE'
              AND d.user_id = u.id
              AND d.status::TEXT = 'ACTIVE'
            "#,
        )
        .bind(token_hash)
        .bind(device_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::database)?
        .ok_or_else(|| AppError::unauthorized("Invalid refresh token or device binding"))
    }

    async fn fetch_active_refresh_binding_with_key(
        &self,
        token_hash: &str,
        device_id: Uuid,
    ) -> Result<RefreshBindingWithKeyRow, AppError> {
        sqlx::query_as::<_, RefreshBindingWithKeyRow>(
            r#"
            SELECT rt.user_id, rt.device_id, d.public_key
            FROM refresh_tokens rt
            INNER JOIN users u ON u.id = rt.user_id
            INNER JOIN devices d ON d.id = rt.device_id
            WHERE rt.token_hash = $1
              AND rt.device_id = $2
              AND rt.revoked = FALSE
              AND rt.expires_at > NOW()
              AND u.deleted_at IS NULL
              AND u.status::TEXT = 'ACTIVE'
              AND d.user_id = u.id
              AND d.status::TEXT = 'ACTIVE'
            "#,
        )
        .bind(token_hash)
        .bind(device_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::database)?
        .ok_or_else(|| AppError::unauthorized("Invalid refresh token or device binding"))
    }

    async fn consume_nonce_challenge(
        &self,
        challenge_id: Uuid,
    ) -> Result<RefreshNonceEntry, AppError> {
        let key = Self::redis_key(challenge_id);
        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection for refresh challenge verification")?;

        let raw: Option<String> = redis::cmd("GETDEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .context("Failed to consume refresh nonce challenge from Redis")?;

        let raw = raw.ok_or_else(|| {
            AppError::unauthorized("Nonce challenge expired or already used (replay rejected)")
        })?;

        serde_json::from_str(&raw)
            .context("Failed to deserialize refresh nonce challenge")
            .map_err(AppError::from)
    }

    fn verify_device_signature(
        &self,
        public_key: &str,
        nonce: &str,
        signature: &str,
    ) -> Result<(), AppError> {
        let public_key_bytes = Self::decode_input_bytes(public_key)
            .ok_or_else(|| AppError::unauthorized("Invalid stored device public key encoding"))?;
        let public_key_bytes: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| AppError::unauthorized("Invalid stored device public key length"))?;

        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|_| AppError::unauthorized("Invalid stored device public key"))?;

        let signature_bytes = Self::decode_input_bytes(signature)
            .ok_or_else(|| AppError::unauthorized("Invalid signature encoding"))?;
        let signature_bytes: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| AppError::unauthorized("Invalid signature length"))?;

        let signature = Signature::from_bytes(&signature_bytes);
        verifying_key
            .verify(nonce.as_bytes(), &signature)
            .map_err(|_| AppError::unauthorized("Invalid device signature"))
    }

    fn issue_access_token(&self, user_id: Uuid, device_id: Uuid) -> Result<String, AppError> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let exp = now + ACCESS_TOKEN_TTL_SECS as i64;
        let claims = AccessTokenClaims {
            sub: user_id.to_string(),
            device_id,
            jti: Uuid::new_v4(),
            exp,
            iat: now,
        };

        jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::internal_error(format!("Failed to issue access token: {e}")))
    }

    pub async fn revoke_refresh_token(&self, token_hash: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = TRUE, revoked_at = NOW()
            WHERE token_hash = $1 AND revoked = FALSE
            "#,
        )
        .bind(token_hash)
        .execute(&self.db)
        .await
        .map_err(AppError::database)?;

        Ok(())
    }

    fn redis_key(challenge_id: Uuid) -> String {
        format!("{NONCE_KEY_PREFIX}:{challenge_id}")
    }

    fn generate_nonce() -> String {
        let mut nonce_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut nonce_bytes);
        URL_SAFE_NO_PAD.encode(nonce_bytes)
    }

    fn decode_input_bytes(input: &str) -> Option<Vec<u8>> {
        if let Ok(decoded) = STANDARD.decode(input) {
            return Some(decoded);
        }

        if let Ok(decoded) = URL_SAFE_NO_PAD.decode(input) {
            return Some(decoded);
        }

        Self::decode_hex(input).ok()
    }

    fn decode_hex(hex_input: &str) -> Result<Vec<u8>, ()> {
        if hex_input.len() % 2 != 0 {
            return Err(());
        }

        let mut out = Vec::with_capacity(hex_input.len() / 2);
        for idx in (0..hex_input.len()).step_by(2) {
            let byte = u8::from_str_radix(&hex_input[idx..idx + 2], 16).map_err(|_| ())?;
            out.push(byte);
        }
        Ok(out)
    }
}
