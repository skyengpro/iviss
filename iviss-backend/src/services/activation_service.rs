use crate::db::RedisPool;
use crate::services::sms_provider::SmsProvider;
use anyhow::{Context, Result};
use deadpool_redis::redis::AsyncCommands;
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const ACTIVATION_TTL_SECS: u64 = 600; // 10 minutes
const MAX_ATTEMPTS: u8 = 5;
const DEFAULT_KEY_PREFIX: &str = "activation";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ActivationEntry {
    pub(crate) code_hash: String,
    pub(crate) attempts: u8,
}

impl ActivationEntry {
    fn redis_key(prefix: &str, user_id: &Uuid) -> String {
        format!("{}:{}", prefix, user_id)
    }
}

pub struct ActivationService {
    pub (crate) redis: RedisPool,
    pub (crate) sms: Arc<dyn SmsProvider>,
    pepper: String,
    prefix: &'static str,
}

impl ActivationService {
    pub fn new(redis: RedisPool, sms: Arc<dyn SmsProvider>, pepper: String) -> Self {
        Self::new_with_prefix(redis, sms, pepper, DEFAULT_KEY_PREFIX)
    }

    /// used by OtpService
    pub fn new_with_prefix(
        redis: RedisPool,
        sms: Arc<dyn SmsProvider>,
        pepper: String,
        prefix: &'static str,
    ) -> Self {
        Self { redis, sms, pepper, prefix }
    }

    /// Generates a 6-digit code, stores its hash in Redis, returns the plain code
    pub async fn generate_and_store(&self, user_id: &Uuid) -> Result<String> {
        let code = self.generate_code();
        let code_hash = self.hash_code(&code);

        let entry = ActivationEntry {
            code_hash,
            attempts: 0,
        };

        let key = ActivationEntry::redis_key(self.prefix, user_id);
        let value =
            serde_json::to_string(&entry).context("Failed to serialize activation entry")?;

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        // Store with TTL 10 minutes — replaces any existing entry
        conn.set_ex::<_, _, ()>(&key, value, ACTIVATION_TTL_SECS)
            .await
            .context("Failed to store activation code in Redis")?;

        Ok(code)
    }

    /// Validates a code submitted by the agent — returns Ok(()) if valid
    pub async fn validate(&self, user_id: &Uuid, submitted_code: &str) -> Result<()> {
        let key = ActivationEntry::redis_key(self.prefix, user_id);
        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        // Retrieve the entry
        let (raw, ttl): (Option<String>, i64) = redis::pipe()
            .get(&key)
            .ttl(&key)
            .query_async(&mut conn)
            .await
            .context("Failed to fetch activation entry")?;

        let raw = raw.ok_or_else(|| anyhow::anyhow!("Activation code expired or not found"))?;

        let remaining_ttl = if ttl > 0 {
            ttl as u64
        } else {
            return Err(anyhow::anyhow!("Activation code expired"));
        };

        let mut entry: ActivationEntry =
            serde_json::from_str(&raw).context("Failed to deserialize activation entry")?;

        // Check number of attempts
        if entry.attempts >= MAX_ATTEMPTS {
            // Delete the key
            conn.del::<_, ()>(&key).await.ok();
            return Err(anyhow::anyhow!(
                "Max attempts reached — activation code invalidated"
            ));
        }

        let submitted_hash = self.hash_code(submitted_code);

        if submitted_hash != entry.code_hash {
            // Increment only on mismatch
            entry.attempts += 1;

            if entry.attempts >= MAX_ATTEMPTS {
                conn.del::<_, ()>(&key).await.ok();
                return Err(anyhow::anyhow!(
                    "Max attempts reached — activation code invalidated"
                ));
            }

            let updated = serde_json::to_string(&entry)?;
            conn.set_ex::<_, _, ()>(&key, updated, remaining_ttl)
                .await
                .context("Failed to update attempts")?;

            return Err(anyhow::anyhow!(
                "Invalid activation code — {} attempt(s) remaining",
                MAX_ATTEMPTS - entry.attempts
            ));
        }

        // Success — delete the key
        conn.del::<_, ()>(&key).await.ok();

        Ok(())
    }

    /// Generate a 6-digit code
    pub(crate) fn generate_code(&self) -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(0..=999_999);
        format!("{:06}", code)
    }

    /// HMAC-SHA256 hash of the plain code
    pub(crate) fn hash_code(&self, code: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.pepper.as_bytes()).expect("HMAC accepts any key size");
        mac.update(code.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    }

    /// Generate a code, store it, and send via SMS
    pub async fn generate_and_send(&self, user_id: &Uuid, phone_number: &str) -> Result<()> {
        let code = self.generate_and_store(user_id).await?;

        let message = format!(
            "Votre code d'activation IVISS est : {}. Valide 10 minutes.",
            code
        );

        self.sms.send_sms(phone_number, &message).await?;

        Ok(())
    }
}
