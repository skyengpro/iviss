use crate::app_cache::{AppCache, OtpEntry};
use crate::errors::AppError;
use crate::services::sms_provider::SmsProvider;
use crate::services::email_provider::EmailProvider;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub const OTP_TTL_SECS: u64 = 300; // 5 minutes — absolute, not sliding
const MAX_ATTEMPTS: u8 = 5;
const RATE_LIMIT_MAX: u32 = 3; // max OTP requests per window

pub struct OtpService {
    app_cache: Arc<AppCache>,
    sms: Arc<dyn SmsProvider>,
    email: Arc<dyn EmailProvider>,
    use_email: bool,
    pepper: String,
}

impl OtpService {

    pub fn new(
        app_cache: Arc<AppCache>,
        sms: Arc<dyn SmsProvider>,
        email: Arc<dyn EmailProvider>,
        pepper: String,
        use_email: bool,
    ) -> Self {
        Self {
            app_cache,
            sms,
            email,
            use_email,
            pepper,
        }
    }

    /// Check rate limit, generate OTP, store in Moka cache and send via SMS

    /// Check rate limit, generate OTP, store in Moka cache and send via SMS or Email
    pub async fn request_otp(&self, user_id: &Uuid, contact: &str) -> Result<(), AppError> {
        self.check_rate_limit(contact).await?;

        let code = self.generate_code();
        let code_hash = self.hash_code(&code);

        // Print OTP to console for development/debugging
        println!(
            "\n┌─────────────────────────────────────┐\
             \n│  OTP CODE                           │\
             \n│  User  : {:<28}│\
             \n│  Contact : {:<28}│\
             \n│  Code  : {:<28}│\
             \n└─────────────────────────────────────┘\n",
            user_id.to_string(),
            contact,
            code,
        );

        let entry = OtpEntry {
            code_hash,
            attempts: 0,
            expires_at: std::time::Instant::now() + std::time::Duration::from_secs(OTP_TTL_SECS),
        };

        // Replaces any existing OTP — TTL is absolute from this point
        self.app_cache.otp_store.insert(*user_id, entry).await;

        if self.use_email {
            self.email
                .send_email(contact, "agent", &code)
                .await
                .map_err(AppError::Internal)?;
        } else {
            let message = format!("Your IVISS login code is: {code}. Valid for 5 minutes.");
            self.sms
                .send_sms(contact, &message)
                .await
                .map_err(AppError::Internal)?;
        }

        info!(target: "otp", user_id = %user_id, "OTP generated and sent");
        Ok(())
    }

    /// Validate a submitted OTP — deletes the entry on success (single use)
    pub async fn validate_otp(&self, user_id: &Uuid, submitted_code: &str) -> Result<(), AppError> {
        let submitted_code = submitted_code.trim();
        if submitted_code.len() != 6 || !submitted_code.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::bad_request("Invalid OTP format"));
        }
        let otp_cache = &self.app_cache.otp_store;

        let mut entry: OtpEntry = otp_cache
            .get(user_id)
            .await
            .ok_or_else(|| AppError::unauthorized("OTP expired or not found"))?;

        if entry.attempts >= MAX_ATTEMPTS {
            otp_cache.invalidate(user_id).await;
            return Err(AppError::unauthorized(
                "Max attempts reached — OTP invalidated",
            ));
        }

        let submitted_hash = self.hash_code(submitted_code);

        if submitted_hash != entry.code_hash {
            entry.attempts += 1;

            // Preserve absolute TTL — do not reset expiration
            otp_cache.insert(*user_id, entry.clone()).await;

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
        otp_cache.invalidate(user_id).await;
        info!(target: "otp", user_id = %user_id, "OTP validated successfully");
        Ok(())
    }

    /// Rate limit — max 3 OTP requests per phone number per 10 minutes
    async fn check_rate_limit(&self, phone: &str) -> Result<(), AppError> {
        let key = phone.to_string();
        let count = self.app_cache.rate_limit.get(&key).await.unwrap_or(0);

        if count >= RATE_LIMIT_MAX {
            warn!(target: "otp", phone = %phone, count = count, "Rate limit exceeded");
            return Err(AppError::too_many_requests(
                "Too many OTP requests — try again later",
            ));
        }
        // Set TTL only on the first request — absolute window,
        self.app_cache.rate_limit.insert(key, count + 1).await;

        Ok(())
    }

    fn generate_code(&self) -> String {
        let code: u32 = rand::thread_rng().gen_range(0..=999_999);
        format!("{code:06}")
    }

    fn hash_code(&self, code: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.pepper.as_bytes()).expect("HMAC accepts any key size");
        mac.update(code.as_bytes());
        let finalize = mac.finalize().into_bytes();
        format!("{finalize:x}")
    }
}
