use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use moka::future::Cache;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info};

///  SMS provider abstraction
#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<()>;
}

pub struct MockSmsProvider;

#[async_trait]
impl SmsProvider for MockSmsProvider {
    async fn send_sms(&self, _phone_number: &str, _message: &str) -> Result<()> {
        Ok(())
    }
}

/// Credentials configuration for SMS providers
/// Allows easy switching between providers via environment variables
#[derive(Clone, Debug)]
pub enum SmsProviderCredentials {
    /// Vonage Messages API credentials
    Vonage {
        api_key: String,
        api_secret: String,
    },
    /// Twilio SMS API credentials
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String,
    },
    /// Orange Cameroun SMS API credentials
    Orange {
        client_id: String,
        client_secret: String,
        sender_number: String,
    },
    Mock,
}

// ─────────────────────────────────────────
// Twilio — provider
// ─────────────────────────────────────────

pub struct TwilioSmsProvider {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
    pub client: reqwest::Client,
}

impl TwilioSmsProvider {
    pub fn new(account_sid: String, auth_token: String, from_number: String) -> Self {
        Self {
            account_sid,
            auth_token,
            from_number,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SmsProvider for TwilioSmsProvider {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let params = [
            ("To", phone_number),
            ("From", &self.from_number),
            ("Body", message),
        ];
        info!(
            target: "sms",
            phone = %phone_number,
            message = %message,
            "Sending SMS via Twilio"
        );
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Twilio error — status: {status}, body: {body}"
            ));
        }

        info!(
            target: "sms",
            phone = %phone_number,
            "SMS sent successfully via Twilio"
        );

        Ok(())
    }
}

// ─────────────────────────────────────────
// Vonage — provider
// ─────────────────────────────────────────
pub struct VonageSmsProvider {
    pub vonage_api_key: String,
    pub vonage_api_secret: String,
    pub client: reqwest::Client,
}

impl VonageSmsProvider {
    pub fn new(vonage_api_key: String, vonage_api_secret: String) -> Self {
        Self {
            vonage_api_key,
            vonage_api_secret,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SmsProvider for VonageSmsProvider {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<()> {
        let url = "https://api.nexmo.com/v1/messages";

        let body = serde_json::json!({
            "to": phone_number,
            "from": "IVISS",
            "channel": "sms",
            "message_type": "text",
            "text": message,
        });

        info!(
            target: "sms",
            phone = %phone_number,
            message = %message,
            "Sending SMS via Vonage"
        );

        let response = self
            .client
            .post(url)
            .basic_auth(&self.vonage_api_key, Some(&self.vonage_api_secret))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let response_body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Vonage error — status: {status}, body: {response_body}"
            ));
        }

        info!(
            target: "sms",
            phone = %phone_number,
            "SMS sent successfully via Vonage"
        );

        Ok(())
    }
}

// ─────────────────────────────────────────
// Orange Cameroun — provider
// ─────────────────────────────────────────

const ORANGE_TOKEN_URL: &str = "https://api.orange.com/oauth/v3/token";
const ORANGE_SMS_BASE_URL: &str = "https://api.orange.com/smsmessaging/v1";
const ORANGE_TOKEN_CACHE_TTL_SECS: u64 = 3300; // 55 minutes (token lasts 60 min)
const ORANGE_RATE_LIMIT_MILLIS: u64 = 200; // 5 SMS per second max

pub struct OrangeSmsProvider {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    sender_number: String,
    token_cache: Cache<String, String>,
    rate_limiter: Mutex<Instant>,
}

#[derive(Deserialize)]
struct OrangeTokenResponse {
    access_token: String,
}

impl OrangeSmsProvider {
    pub fn new(client_id: String, client_secret: String, sender_number: String) -> Self {
        let token_cache = Cache::builder()
            .time_to_live(Duration::from_secs(ORANGE_TOKEN_CACHE_TTL_SECS))
            .build();

        Self {
            client: reqwest::Client::new(),
            client_id,
            client_secret,
            sender_number,
            token_cache,
            rate_limiter: Mutex::new(Instant::now()),
        }
    }

    async fn get_valid_token(&self) -> Result<String> {
        // Check cache first
        if let Some(token) = self.token_cache.get(&"token".to_string()).await {
            debug!("Using cached Orange OAuth token");
            return Ok(token);
        }

        // Fetch new token
        debug!("Fetching new Orange OAuth token");
        let auth_header = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", self.client_id, self.client_secret))
        );

        let response = self
            .client
            .post(ORANGE_TOKEN_URL)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body("grant_type=client_credentials")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch Orange token: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Orange token API returned {}: {}",
                status,
                body
            ));
        }

        let token_res: OrangeTokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse Orange token response: {}", e))?;

        let access_token = token_res.access_token.clone();

        // Cache the token
        self.token_cache
            .insert("token".to_string(), token_res.access_token)
            .await;

        Ok(access_token)
    }

    fn normalize_msisdn(&self, msisdn: &str) -> String {
        let digits: String = msisdn.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.starts_with("237") && digits.len() == 12 {
            format!("tel:+{}", digits)
        } else if digits.len() == 9 {
            format!("tel:+237{}", digits)
        } else {
            format!("tel:+{}", digits)
        }
    }

    async fn apply_rate_limit(&self) {
        let mut last_send = self.rate_limiter.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_send);
        let wait_time = Duration::from_millis(ORANGE_RATE_LIMIT_MILLIS);
        if elapsed < wait_time {
            tokio::time::sleep(wait_time - elapsed).await;
        }
        *last_send = Instant::now();
    }
}

#[async_trait]
impl SmsProvider for OrangeSmsProvider {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<()> {
        let token = self.get_valid_token().await?;
        let normalized_to = self.normalize_msisdn(phone_number);
        let normalized_from = self.normalize_msisdn(&self.sender_number);

        // Apply rate limiting (5 SMS/sec max)
        self.apply_rate_limit().await;

        let url = format!(
            "{}/outbound/{}/requests",
            ORANGE_SMS_BASE_URL,
            urlencoding::encode(&normalized_from)
        );

        let payload = serde_json::json!({
            "outboundSMSMessageRequest": {
                "address": normalized_to,
                "senderAddress": normalized_from,
                "outboundSMSTextMessage": {
                    "message": message
                }
            }
        });

        info!(
            target: "sms",
            phone = %phone_number,
            "Sending SMS via Orange Cameroun"
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to contact Orange API: {}", e))?;

        let status = response.status();
        if status.is_success() {
            info!(
                target: "sms",
                phone = %phone_number,
                "SMS sent successfully via Orange Cameroun"
            );
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Orange API error ({}): {}", status, body))
        }
    }
}

impl SmsProviderCredentials {
    /// Get the provider name
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Vonage { .. } => "vonage",
            Self::Twilio { .. } => "twilio",
            Self::Orange { .. } => "orange",
            Self::Mock => "mock",
        }
    }

    /// Create the appropriate SMS provider instance
    pub fn provider(&self) -> Arc<dyn SmsProvider> {
        match self {
            Self::Vonage {
                api_key,
                api_secret,
            } => {
                info!("Using Vonage SMS provider");
                Arc::new(VonageSmsProvider::new(api_key.clone(), api_secret.clone()))
            }
            Self::Twilio {
                account_sid,
                auth_token,
                from_number,
            } => {
                info!("Using Twilio SMS provider");
                Arc::new(TwilioSmsProvider::new(
                    account_sid.clone(),
                    auth_token.clone(),
                    from_number.clone(),
                ))
            }
            Self::Orange {
                client_id,
                client_secret,
                sender_number,
            } => {
                info!("Using Orange Cameroun SMS provider");
                Arc::new(OrangeSmsProvider::new(
                    client_id.clone(),
                    client_secret.clone(),
                    sender_number.clone(),
                ))
            }
            Self::Mock => Arc::new(MockSmsProvider),
        }
    }
}
