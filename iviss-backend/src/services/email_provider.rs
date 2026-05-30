use anyhow::Result;
use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Arc;
use tracing::{info, warn};

/// Email provider abstraction
#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send_email(&self, to: &str, user_role: &str, password: &str) -> Result<()>;
}

pub struct MockEmailProvider;

#[async_trait]
impl EmailProvider for MockEmailProvider {
    async fn send_email(&self, to: &str, _user_role: &str, password: &str) -> Result<()> {
        warn!(
            target: "email",
            to = %to,
            password = %password,
            "[MOCK EMAIL] — email not actually sent"
        );
        Ok(())
    }
}

/// Credentials configuration for email providers
#[derive(Clone, Debug)]
pub enum EmailProviderCredentials {
    /// Resend.com API credentials
    Resend { api_key: String, from_email: String },
    /// Lettre SMTP credentials
    Lettre {
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        from_email: String,
    },
    /// Mock provider for development/testing
    Mock,
}

impl EmailProviderCredentials {
    /// Check if credentials are mock/empty
    pub fn is_mock(&self) -> bool {
        matches!(self, Self::Mock)
    }

    /// Get the provider name
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Resend { .. } => "resend",
            Self::Lettre { .. } => "lettre",
            Self::Mock => "mock",
        }
    }

    /// Create the appropriate email provider instance
    pub fn provider(&self) -> Arc<dyn EmailProvider> {
        match self {
            Self::Resend {
                api_key,
                from_email,
            } => {
                info!("Using Resend email provider");
                Arc::new(ResendEmailProvider::new(
                    api_key.clone(),
                    from_email.clone(),
                ))
            }
            Self::Mock => {
                info!("Using Mock email provider (logs to console)");
                Arc::new(MockEmailProvider)
            }
            Self::Lettre {
                smtp_host,
                smtp_port,
                smtp_username,
                smtp_password,
                from_email,
            } => {
                info!("Using Lettre SMTP email provider");
                Arc::new(LettreEmailProvider::new(
                    smtp_host.clone(),
                    *smtp_port,
                    smtp_username.clone(),
                    smtp_password.clone(),
                    from_email.clone(),
                ))
            }
        }
    }
}

// ─────────────────────────────────────────
// Resend — provider
// ─────────────────────────────────────────

pub struct ResendEmailProvider {
    pub api_key: String,
    pub from_email: String,
    pub client: reqwest::Client,
}

impl ResendEmailProvider {
    pub fn new(api_key: String, from_email: String) -> Self {
        Self {
            api_key,
            from_email,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EmailProvider for ResendEmailProvider {
    async fn send_email(&self, to: &str, user_role: &str, password: &str) -> Result<()> {
        let url = "https://api.resend.com/emails";

        // Compose email body based on role
        let email_body = match user_role {
            "org_admin" => format!(
                r#"<h1>Welcome to IVISS</h1>
        <p>Your org admin account has been created.</p>
        <p><strong>Email:</strong> {}</p>
        <p><strong>Temporary Password:</strong> {}</p>
        <p>You must change your password on first login.</p>"#,
                to, password
            ),
            "agent" => format!(
                r#"<h1>IVISS Authentication</h1>
        <p>Your daily login code is:</p>
        <p><strong>{}</strong></p>
        <p>Valid for 5 minutes.</p>"#,
                password
            ),
            _ => format!("<p>You are not identified.</p>"),
        };
        let payload = serde_json::json!({
            "from": self.from_email,
            "to": to,
            "subject": "IVISS AUTHENTICATION",
            "html": email_body,
        });

        info!(
            target: "email",
            to = %to,
            subject = "IVISS AUTHENTICATION",
            "Sending email via Resend"
        );

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let response_body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Resend Email Service error — status: {status}, body: {response_body}"
            ));
        }

        info!(
            target: "email",
            to = %to,
            "Email sent successfully via Resend"
        );

        Ok(())
    }
}

// ─────────────────────────────────────────
// Lettre — SMTP provider
// ─────────────────────────────────────────

pub struct LettreEmailProvider {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
}

impl LettreEmailProvider {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        from_email: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
        }
    }

    fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        // Use STARTTLS for port 587, TLS for 465
        let transport = if self.smtp_port == 587 {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_host)
                .map_err(|e| anyhow::anyhow!("Invalid SMTP host: {e}"))?
                .port(self.smtp_port)
                .credentials(creds)
                .build()
        } else {
            // Port 465 or other - use TLS wrapper
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host)
                .map_err(|e| anyhow::anyhow!("Invalid SMTP host: {e}"))?
                .port(self.smtp_port)
                .credentials(creds)
                .build()
        };

        Ok(transport)
    }
}

#[async_trait]
impl EmailProvider for LettreEmailProvider {
    async fn send_email(&self, to: &str, user_role: &str, password: &str) -> Result<()> {
        let email_body = match user_role {
            "org_admin" => format!(
                r#"<h1>Welcome to IVISS</h1>
        <p>Your org admin account has been created.</p>
        <p><strong>Email:</strong> {}</p>
        <p><strong>Temporary Password:</strong> {}</p>
        <p>You must change your password on first login.</p>"#,
                to, password
            ),
            "agent" => format!(
                r#"<h1>IVISS Authentication</h1>
        <p>Your daily login code is:</p>
        <p><strong>{}</strong></p>
        <p>Valid for 5 minutes.</p>"#,
                password
            ),
            _ => format!("<p>You are not identified.</p>"),
        };

        let email = Message::builder()
            .from(
                self.from_email
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid from email: {e}"))?,
            )
            .to(to
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid to email: {e}"))?)
            .subject("IVISS AUTHENTICATION")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(email_body)
            .map_err(|e| anyhow::anyhow!("Failed to build email: {e}"))?;

        info!(
            target: "email",
            to = %to,
            subject = "IVISS AUTHENTICATION",
            "Sending email via Lettre SMTP"
        );

        let transport = self.build_transport()?;

        transport
            .send(email)
            .await
            .map_err(|e| anyhow::anyhow!("Lettre SMTP error: {e}"))?;

        info!(
            target: "email",
            to = %to,
            "Email sent successfully via Lettre SMTP"
        );

        Ok(())
    }
}
