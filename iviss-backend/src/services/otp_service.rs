use crate::db::RedisPool;
use crate::errors::AppError;
use crate::services::sms_provider::SmsProvider;
use deadpool_redis::redis::AsyncCommands;
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const OTP_TTL_SECS: u64 = 300; // 5 minutes — absolute, not sliding
const MAX_ATTEMPTS: u8 = 5;
const RATE_LIMIT_MAX: u64 = 3; // max OTP requests per window
const RATE_LIMIT_WINDOW_SECS: i64 = 600; // 10 minutes

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OtpEntry {
    code_hash: String,
    attempts: u8,
}

pub struct OtpService {
    redis: RedisPool,
    sms: Arc<dyn SmsProvider>,
    pepper: String,
}

impl OtpService {
    pub fn new(redis: RedisPool, sms: Arc<dyn SmsProvider>, pepper: String) -> Self {
        Self { redis, sms, pepper }
    }

    /// Check rate limit, generate OTP, store in Redis and send via SMS
    pub async fn request_otp(&self, user_id: &Uuid, phone: &str) -> Result<(), AppError> {
        self.check_rate_limit(phone).await?;

        let code = self.generate_code();
        let code_hash = self.hash_code(&code);

        // Print OTP to console for development/debugging
        println!(
            "\n┌─────────────────────────────────────┐\
             \n│  OTP CODE                           │\
             \n│  User  : {:<28}│\
             \n│  Phone : {:<28}│\
             \n│  Code  : {:<28}│\
             \n└─────────────────────────────────────┘\n",
            user_id.to_string(),
            phone,
            code,
        );

        let entry = OtpEntry {
            code_hash,
            attempts: 0,
        };
        let value = serde_json::to_string(&entry)
            .map_err(|e| AppError::internal_error(format!("OTP serialization failed: {e}")))?;

        let key = Self::otp_key(user_id);
        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| AppError::internal_error(format!("Redis connection failed: {e}")))?;

        // Replaces any existing OTP — TTL is absolute from this point
        conn.set_ex::<_, _, ()>(&key, value, OTP_TTL_SECS)
            .await
            .map_err(|e| AppError::internal_error(format!("Redis SET failed: {e}")))?;

        let message = format!("Your IVISS login code is: {}. Valid for 5 minutes.", code);
        self.sms
            .send_sms(phone, &message)
            .await
            .map_err(AppError::Internal)?;

        info!(target: "otp", user_id = %user_id, "OTP generated and sent");
        Ok(())
    }

    /// Validate a submitted OTP — deletes the entry on success (single use)
    pub async fn validate_otp(&self, user_id: &Uuid, submitted_code: &str) -> Result<(), AppError> {
        let submitted_code = submitted_code.trim();
        if submitted_code.len() != 6 || !submitted_code.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::bad_request("Invalid OTP format"));
        }

        let key = Self::otp_key(user_id);
        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| AppError::internal_error(format!("Redis connection failed: {e}")))?;

        // Atomic read — preserve remaining TTL to avoid sliding window
        let (raw, ttl): (Option<String>, i64) = redis::pipe()
            .get(&key)
            .ttl(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::internal_error(format!("Redis pipeline failed: {e}")))?;

        let raw = raw.ok_or_else(|| AppError::unauthorized("OTP expired or not found"))?;

        if ttl <= 0 {
            return Err(AppError::unauthorized("OTP expired"));
        }
        let remaining_ttl = ttl as u64;

        let mut entry: OtpEntry = serde_json::from_str(&raw)
            .map_err(|e| AppError::internal_error(format!("OTP deserialization failed: {e}")))?;

        if entry.attempts >= MAX_ATTEMPTS {
            conn.del::<_, ()>(&key).await.ok();
            return Err(AppError::unauthorized(
                "Max attempts reached — OTP invalidated",
            ));
        }

        let submitted_hash = self.hash_code(submitted_code);

        if submitted_hash != entry.code_hash {
            entry.attempts += 1;

            if entry.attempts >= MAX_ATTEMPTS {
                conn.del::<_, ()>(&key).await.ok();
                warn!(target: "otp", user_id = %user_id, "OTP invalidated: max attempts reached");
                return Err(AppError::unauthorized(
                    "Max attempts reached — OTP invalidated",
                ));
            }

            // Preserve absolute TTL — do not reset expiration
            let updated = serde_json::to_string(&entry).unwrap_or_default();
            conn.set_ex::<_, _, ()>(&key, updated, remaining_ttl)
                .await
                .ok();

            warn!(
                target: "otp",
                user_id = %user_id,
                attempts = entry.attempts,
                "Invalid OTP"
            );
            return Err(AppError::unauthorized(format!(
                "Invalid OTP — {} attempt(s) remaining",
                MAX_ATTEMPTS - entry.attempts
            )));
        }

        // Success — single use: delete immediately
        conn.del::<_, ()>(&key).await.ok();
        info!(target: "otp", user_id = %user_id, "OTP validated successfully");
        Ok(())
    }

    /// Rate limit — max 3 OTP requests per phone number per 10 minutes
    async fn check_rate_limit(&self, phone: &str) -> Result<(), AppError> {
        let key = format!("rate_limit:otp_request:{}", phone);
        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| AppError::internal_error(format!("Redis connection failed: {e}")))?;

        let count: u64 = conn
            .incr(&key, 1u64)
            .await
            .map_err(|e| AppError::internal_error(format!("Redis INCR failed: {e}")))?;

        // Set TTL only on the first request — absolute window,
        if count == 1 {
            conn.expire::<_, ()>(&key, RATE_LIMIT_WINDOW_SECS)
                .await
                .ok();
        }

        if count > RATE_LIMIT_MAX {
            warn!(target: "otp", phone = %phone, count = count, "Rate limit exceeded");
            return Err(AppError::too_many_requests(
                "Too many OTP requests — try again later",
            ));
        }

        Ok(())
    }

    fn otp_key(user_id: &Uuid) -> String {
        format!("user_otp:{}", user_id)
    }

    fn generate_code(&self) -> String {
        let code: u32 = rand::thread_rng().gen_range(0..=999_999);
        format!("{:06}", code)
    }

    fn hash_code(&self, code: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.pepper.as_bytes()).expect("HMAC accepts any key size");
        mac.update(code.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    }
}
