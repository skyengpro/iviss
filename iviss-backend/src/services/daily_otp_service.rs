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

/// Daily OTP lives for 5 minutes
const DAILY_OTP_TTL_SECS: u64 = 300;
/// Max verification attempts before the OTP is invalidated
const MAX_ATTEMPTS: u8 = 5;
/// Redis key namespace
const KEY_PREFIX: &str = "daily_otp";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DailyOtpEntry {
    pub(crate) code_hash: String,
    pub(crate) attempts: u8,
}

impl DailyOtpEntry {
    fn redis_key(user_id: &Uuid) -> String {
        format!("{}:{}", KEY_PREFIX, user_id)
    }
}

pub struct DailyOtpService {
    redis: RedisPool,
    sms: Arc<dyn SmsProvider>,
    pepper: String,
}

impl DailyOtpService {
    pub fn new(redis: RedisPool, sms: Arc<dyn SmsProvider>, pepper: String) -> Self {
        Self { redis, sms, pepper }
    }

    /// Generates a 6-digit OTP, stores its HMAC-SHA256 hash in Redis, returns the plain code
    pub async fn generate_and_store(&self, user_id: &Uuid) -> Result<String> {
        let code = self.generate_code();
        let code_hash = self.hash_code(&code);

        let entry = DailyOtpEntry {
            code_hash,
            attempts: 0,
        };

        let key = DailyOtpEntry::redis_key(user_id);
        let value =
            serde_json::to_string(&entry).context("Failed to serialize daily OTP entry")?;

        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        // Store with 5-minute TTL — replaces any existing OTP for this user
        conn.set_ex::<_, _, ()>(&key, value, DAILY_OTP_TTL_SECS)
            .await
            .context("Failed to store daily OTP in Redis")?;

        Ok(code)
    }

    /// Validates a submitted OTP — returns Ok(()) if valid
    pub async fn validate(&self, user_id: &Uuid, submitted_otp: &str) -> Result<()> {
        let key = DailyOtpEntry::redis_key(user_id);
        let mut conn = self
            .redis
            .get()
            .await
            .context("Failed to get Redis connection")?;

        // Retrieve the entry and its remaining TTL
        let (raw, ttl): (Option<String>, i64) = redis::pipe()
            .get(&key)
            .ttl(&key)
            .query_async(&mut conn)
            .await
            .context("Failed to fetch daily OTP entry")?;

        let raw = raw.ok_or_else(|| anyhow::anyhow!("Daily OTP expired or not found"))?;

        let remaining_ttl = if ttl > 0 {
            ttl as u64
        } else {
            return Err(anyhow::anyhow!("Daily OTP expired"));
        };

        let mut entry: DailyOtpEntry =
            serde_json::from_str(&raw).context("Failed to deserialize daily OTP entry")?;

        // Check number of attempts
        if entry.attempts >= MAX_ATTEMPTS {
            conn.del::<_, ()>(&key).await.ok();
            return Err(anyhow::anyhow!(
                "Max attempts reached — daily OTP invalidated"
            ));
        }

        let submitted_hash = self.hash_code(submitted_otp);

        if submitted_hash != entry.code_hash {
            // Increment attempt counter
            entry.attempts += 1;

            if entry.attempts >= MAX_ATTEMPTS {
                conn.del::<_, ()>(&key).await.ok();
                return Err(anyhow::anyhow!(
                    "Max attempts reached — daily OTP invalidated"
                ));
            }

            let updated = serde_json::to_string(&entry)?;
            conn.set_ex::<_, _, ()>(&key, updated, remaining_ttl)
                .await
                .context("Failed to update OTP attempts")?;

            return Err(anyhow::anyhow!(
                "Invalid OTP — {} attempt(s) remaining",
                MAX_ATTEMPTS - entry.attempts
            ));
        }

        // Success — consume the OTP
        conn.del::<_, ()>(&key).await.ok();

        Ok(())
    }

    /// Generate a 6-digit numeric code
    pub(crate) fn generate_code(&self) -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(0..=999_999);
        format!("{:06}", code) // zero-padded
    }

    /// HMAC-SHA256 hash of the plain OTP using the application pepper
    pub(crate) fn hash_code(&self, code: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.pepper.as_bytes()).expect("HMAC accepts any key size");
        mac.update(code.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    }

    /// Generate an OTP, store it in Redis, and send it via SMS
    pub async fn generate_and_send(&self, user_id: &Uuid, phone_number: &str) -> Result<()> {
        let code = self.generate_and_store(user_id).await?;

        let message = format!(
            "Votre code de connexion quotidien IVISS est : {}. Valide 5 minutes.",
            code
        );

        self.sms.send_sms(phone_number, &message).await?;

        Ok(())
    }
}
