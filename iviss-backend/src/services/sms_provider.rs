use anyhow::Result;
use async_trait::async_trait;
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

// ─────────────────────────────────────────
// Twilio — production
// ─────────────────────────────────────────

pub struct TwilioSmsProvider {
    account_sid: String,
    auth_token: String,
    from_number: String,
    client: reqwest::Client,
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
                "Twilio error — status: {}, body: {}",
                status,
                body
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
