use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

///  SMS provider abstraction
#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<()>;
}

pub struct MockSmsProvider;

#[async_trait]
impl SmsProvider for MockSmsProvider {
    async fn send_sms(&self, phone_number: &str, message: &str) -> Result<()> {
        // Simulates a random failure in dev to test error handling
        warn!(
            target: "sms",
            phone = %phone_number,
            message = %message,
            "[MOCK SMS] — message not actually sent"
        );
        Ok(())
    }
}

/// Credentials configuration for SMS providers
/// Allows easy switching between providers via environment variables
#[derive(Clone, Debug)]
pub enum SmsProviderCredentials {
    /// Vonage Messages API credentials
    Vonage { api_key: String, api_secret: String },
    /// Twilio SMS API credentials
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String,
    },
    /// Mock provider for development/testing
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

impl Default for SmsProviderCredentials {
    fn default() -> Self {
        Self::Mock
    }
}

impl SmsProviderCredentials {
    /// Check if credentials are mock/empty
    pub fn is_mock(&self) -> bool {
        matches!(self, Self::Mock)
    }

    /// Get the provider name
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Vonage { .. } => "vonage",
            Self::Twilio { .. } => "twilio",
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
            Self::Mock => {
                info!("Using Mock SMS provider (logs OTP to console)");
                Arc::new(MockSmsProvider)
            }
        }
    }
}
