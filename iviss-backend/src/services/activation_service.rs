use crate::db::RedisPool;
use crate::services::sms_provider::SmsProvider;
use anyhow::{Context, Result};
use deadpool_redis::redis::AsyncCommands;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

const ACTIVATION_TTL_SECS: u64 = 600; // 10 minutes
const MAX_ATTEMPTS: u8 = 5;
const KEY_PREFIX: &str = "activation";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ActivationEntry {
    pub(crate) code_hash: String,
    pub(crate) attempts: u8,
}

impl ActivationEntry {
    fn redis_key(user_id: &Uuid) -> String {
        format!("{}:{}", KEY_PREFIX, user_id)
    }
}

pub struct ActivationService {
    redis: RedisPool,
    sms: Arc<dyn SmsProvider>,
}

impl ActivationService {
    pub fn new(redis: RedisPool, sms: Arc<dyn SmsProvider>) -> Self {
        Self { redis, sms }
    }

    /// Generates a 6-digit code, stores its hash in Redis, returns the plain code
    pub async fn generate_and_store(&self, user_id: &Uuid) -> Result<String> {
        let code = self.generate_code();
        let code_hash = Self::hash_code(&code);

        let entry = ActivationEntry {
            code_hash,
            attempts: 0,
        };

        let key = ActivationEntry::redis_key(user_id);
        let value =
            serde_json::to_string(&entry).context("Failed to serialize activation entry")?;

        let mut conn = self.redis.get().await
            .context("Failed to get Redis connection")?;

        // Store with TTL 10 minutes — replaces any existing entry
        conn.set_ex::<_, _, ()>(&key, value, ACTIVATION_TTL_SECS)
            .await
            .context("Failed to store activation code in Redis")?;

        Ok(code)
    }

    /// Validates a code submitted by the agent — returns Ok(()) if valid
    pub async fn validate(&self, user_id: &Uuid, submitted_code: &str) -> Result<()> {
        let key = ActivationEntry::redis_key(user_id);
        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        // Retrieve the entry
        let raw: Option<String> = conn
            .get(&key)
            .await
            .context("Failed to get activation entry from Redis")?;

        let raw = raw.ok_or_else(|| anyhow::anyhow!("Activation code expired or not found"))?;

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

        // Increment attempts before verification
        entry.attempts += 1;
        let updated = serde_json::to_string(&entry).context("Failed to serialize")?;
        conn.set_ex::<_, _, ()>(&key, updated, ACTIVATION_TTL_SECS)
            .await
            .context("Failed to update attempts")?;

        // Verify the hash
        let submitted_hash = Self::hash_code(submitted_code);
        if submitted_hash != entry.code_hash {
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
        format!("{:06}", code) // zero-padded
    }

    /// SHA-256 hash of the plain code
    pub(crate) fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
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
