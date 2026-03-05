use crate::db::redis::RedisPool;
use crate::services::activation_service::ActivationService;
use crate::services::sms_provider::SmsProvider;
use anyhow::{Context, Result};
use deadpool_redis::redis::AsyncCommands;
use std::sync::Arc;
use uuid::Uuid;

const RATE_LIMIT_MAX_REQUESTS: u8 = 3;
const RATE_LIMIT_TTL_SECS: u64 = 600; // 10 minutes window

pub struct OtpService {
    inner: ActivationService,   // delegates all OTP logic to ActivationService
}

impl OtpService {
    pub fn new(redis: RedisPool, sms: Arc<dyn SmsProvider>, pepper: String) -> Self {
        Self {
            inner: ActivationService::new_with_prefix(
                redis.clone(),
                sms,
                pepper,
                "otp",
            ),
        }
    }

    /// Checks rate limit, generates OTP and sends via SMS
    pub async fn request_otp(&self, user_id: &Uuid, phone_number: &str) -> Result<()> {
        // Enforce rate limit before generating
        self.check_rate_limit(phone_number).await?;

        let code = self.inner.generate_and_store(user_id).await?;

        let message = format!(
            "Your daily IVISS login code is: {}. Valid for 10 minutes.",
            code
        );

        self.inner.sms.send_sms(phone_number, &message).await
    }

    /// Delegates OTP validation to ActivationService
    pub async fn validate_otp(&self, user_id: &Uuid, code: &str) -> Result<()> {
        self.inner.validate(user_id, code).await
    }

    /// Rate limit: max 3 requests per phone number per 10 minutes
    async fn check_rate_limit(&self, phone_number: &str) -> Result<()> {
        let key = format!("rate_limit:otp_request:{}", phone_number);
        let mut conn = self.inner.redis.get().await
            .context("Failed to get Redis connection")?;

        // Increment counter
        let count: u8 = conn.incr(&key, 1).await
            .context("Failed to increment rate limit counter")?;

        // Set TTL only on first request — preserves absolute window
        if count == 1 {
            let _: ()= conn.expire(&key, RATE_LIMIT_TTL_SECS as i64).await
                .context("Failed to set rate limit TTL")?;
        }

        if count > RATE_LIMIT_MAX_REQUESTS {
            return Err(anyhow::anyhow!(
                "Too many OTP requests — try again in 10 minutes"
            ));
        }

        Ok(())
    }
}