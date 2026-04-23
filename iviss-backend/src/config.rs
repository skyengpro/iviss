pub use crate::services::email_provider::EmailProviderCredentials;
pub use crate::services::sms_provider::SmsProviderCredentials;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::env;
use std::str::FromStr;

/// Application environment type
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Local,
    Staging,
    Production,
}

impl FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "staging" => Ok(Environment::Staging),
            "production" => Ok(Environment::Production),
            _ => Err(anyhow!(
                "Invalid ENVIRONMENT value: '{s}'. Must be one of: local, staging, production"
            )),
        }
    }
}

/// Logging level configuration
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(anyhow!(
                "Invalid LOG_LEVEL value: '{s}'. Must be one of: trace, debug, info, warn, error"
            )),
        }
    }
}

impl LogLevel {
    /// Convert LogLevel to tracing::Level
    pub fn as_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// Application configuration
#[derive(Clone, Debug)]
// jwt and helper methods will be used in future JWT implementation
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub log_level: LogLevel,
    pub jwt_private_key_pem: String,
    pub jwt_public_key_pem: String,
    pub environment: Environment,
    // SMS
    pub sms_credentials: SmsProviderCredentials,
    // Email
    pub email_credentials: EmailProviderCredentials,
    pub activation_code_pepper: String,
    // Bootstrap admin — used only at first startup if no admin exists
    pub admin_bootstrap_email: Option<String>,
    pub admin_bootstrap_password: Option<String>,
    pub admin_bootstrap_phone: Option<String>,
    pub admin_bootstrap_username: Option<String>,
}

impl Config {
    /// Load configuration from environment variables with fail-fast validation
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        // Load and validate DATABASE_URL (critical)
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        if database_url.trim().is_empty() {
            return Err(anyhow!("DATABASE_URL cannot be empty"));
        }

        // Load and validate JWT_PRIVATE_KEY_PEM (critical)
        let jwt_private_key_pem = env::var("JWT_PRIVATE_KEY_PEM")
            .context("JWT_PRIVATE_KEY_PEM must be set")?
            .replace("\\n", "\n");
        if jwt_private_key_pem.trim().is_empty() {
            return Err(anyhow!("JWT_PRIVATE_KEY_PEM cannot be empty"));
        }

        let jwt_public_key_pem = env::var("JWT_PUBLIC_KEY_PEM")
            .context("JWT_PUBLIC_KEY_PEM must be set")?
            .replace("\\n", "\n");
        if jwt_public_key_pem.trim().is_empty() {
            return Err(anyhow!("JWT_PUBLIC_KEY_PEM cannot be empty"));
        }

        // Load SERVER_HOST with default
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        // Load and validate SERVER_PORT
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .context("SERVER_PORT must be a valid port number (0-65535)")?;

        // Load and validate LOG_LEVEL
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string())
            .parse::<LogLevel>()
            .context("Failed to parse LOG_LEVEL")?;

        // Load and validate ENVIRONMENT
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "local".to_string())
            .parse::<Environment>()
            .context("Failed to parse ENVIRONMENT")?;

        // SMS Provider configuration
        let sms_provider = env::var("SMS_PROVIDER").context("SMS_PROVIDER must be set")?;
        let sms_credentials = Self::get_sms_provider_credentials(&sms_provider)
            .context("Failed to configure SMS provider")?;

        // Email Provider configuration
        let email_provider = env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "mock".to_string());
        let email_credentials = Self::get_email_provider_credentials(&email_provider);

        let activation_code_pepper =
            env::var("ACTIVATION_CODE_PEPPER").context("ACTIVATION_CODE_PEPPER must be set")?;

        if activation_code_pepper.len() < 32 {
            return Err(anyhow!(
                "ACTIVATION_CODE_PEPPER must be at least 32 characters"
            ));
        }

        // Bootstrap admin — all optional, seed is skipped if any is missing
        let admin_bootstrap_email = env::var("ADMIN_BOOTSTRAP_EMAIL").ok();
        let admin_bootstrap_password = env::var("ADMIN_BOOTSTRAP_PASSWORD").ok();
        let admin_bootstrap_phone = env::var("ADMIN_BOOTSTRAP_PHONE").ok();
        let admin_bootstrap_username = env::var("ADMIN_BOOTSTRAP_USERNAME").ok();

        Ok(Self {
            database_url,
            server_host,
            server_port,
            log_level,
            jwt_private_key_pem,
            jwt_public_key_pem,
            environment,
            sms_credentials,
            email_credentials,
            activation_code_pepper,
            admin_bootstrap_email,
            admin_bootstrap_password,
            admin_bootstrap_phone,
            admin_bootstrap_username,
        })
    }

    fn get_sms_provider_credentials(sms_provider: &str) -> Result<SmsProviderCredentials> {
        match sms_provider.to_lowercase().as_str() {
            "vonage" => {
                let api_key = env::var("VONAGE_API_KEY").unwrap_or_default();
                let api_secret = env::var("VONAGE_API_SECRET").unwrap_or_default();

                if api_key.trim().is_empty() || api_secret.trim().is_empty() {
                    eprintln!(
                        "SMS_PROVIDER=vonage but VONAGE_API_KEY/VONAGE_API_SECRET are not set"
                    );
                }
                Ok(SmsProviderCredentials::Vonage {
                    api_key,
                    api_secret,
                })
            }
            "twilio" => {
                let account_sid = env::var("TWILIO_ACCOUNT_SID").unwrap_or_default();
                let auth_token = env::var("TWILIO_AUTH_TOKEN").unwrap_or_default();
                let from_number = env::var("TWILIO_FROM_NUMBER").unwrap_or_default();

                if account_sid.trim().is_empty()
                    || auth_token.trim().is_empty()
                    || from_number.trim().is_empty()
                {
                    eprintln!(
                        "SMS_PROVIDER=twilio but TWILIO_ACCOUNT_SID/TWILIO_AUTH_TOKEN/TWILIO_FROM_NUMBER are not set"
                    );
                }
                Ok(SmsProviderCredentials::Twilio {
                    account_sid,
                    auth_token,
                    from_number,
                })
            }
            "orange" => {
                let client_id = env::var("ORANGE_CLIENT_ID").unwrap_or_default();
                let client_secret = env::var("ORANGE_CLIENT_SECRET").unwrap_or_default();
                let sender_number = env::var("ORANGE_SENDER_NUMBER")
                    .unwrap_or_else(|_| "+237000000000".to_string());

                let orange_creds_invalid =
                    client_id.trim().is_empty() || client_secret.trim().is_empty();

                if orange_creds_invalid {
                    eprintln!(
                        "SMS_PROVIDER=orange but ORANGE_CLIENT_ID/ORANGE_CLIENT_SECRET are not set"
                    );
                }
                Ok(SmsProviderCredentials::Orange {
                    client_id,
                    client_secret,
                    sender_number,
                })
            }
            other => Err(anyhow!(
                "Invalid SMS_PROVIDER value: '{other}'. Must be one of: vonage, twilio, orange"
            )),
        }
    }

    fn get_email_provider_credentials(email_provider: &str) -> EmailProviderCredentials {
        match email_provider.to_lowercase().as_str() {
            "resend" => {
                let api_key = env::var("RESEND_API_KEY").unwrap_or_else(|_| "mock".to_string());
                let from_email = env::var("RESEND_FROM_EMAIL")
                    .unwrap_or_else(|_| "mock@example.com".to_string());
                EmailProviderCredentials::Resend {
                    api_key,
                    from_email,
                }
            }
            "lettre" | "smtp" => {
                let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
                let smtp_port = env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse::<u16>()
                    .unwrap_or(587);
                let smtp_username =
                    env::var("SMTP_USERNAME").unwrap_or_else(|_| "user".to_string());
                let smtp_password =
                    env::var("SMTP_PASSWORD").unwrap_or_else(|_| "password".to_string());
                let from_email = env::var("SMTP_FROM_EMAIL")
                    .unwrap_or_else(|_| "noreply@iviss.local".to_string());
                EmailProviderCredentials::Lettre {
                    smtp_host,
                    smtp_port,
                    smtp_username,
                    smtp_password,
                    from_email,
                }
            }
            _ => EmailProviderCredentials::Mock,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Additional validation can be added here
        if self.server_port == 0 {
            // return Err(anyhow!("SERVER_PORT cannot be 0"));
        }
        // Validate SMS provider config in production
        if self.environment == Environment::Production {
            if matches!(&self.sms_credentials, SmsProviderCredentials::Mock) {
                return Err(anyhow!(
                    "Mock SMS provider is not allowed in production environment"
                ));
            }
            if matches!(
                &self.sms_credentials,
                SmsProviderCredentials::Orange {
                    client_id,
                    client_secret,
                    ..
                } if client_id.trim().is_empty()
                    || client_secret.trim().is_empty()
            ) {
                return Err(anyhow!(
                    "ORANGE_CLIENT_ID and ORANGE_CLIENT_SECRET must be set when SMS_PROVIDER=orange"
                ));
            }
        }

        if matches!(
            &self.sms_credentials,
            SmsProviderCredentials::Vonage { api_key, api_secret }
                if api_key.trim().is_empty() || api_secret.trim().is_empty()
        ) {
            return Err(anyhow!(
                "VONAGE_API_KEY and VONAGE_API_SECRET must be set when SMS_PROVIDER=vonage"
            ));
        }

        if matches!(
            &self.sms_credentials,
            SmsProviderCredentials::Twilio {
                account_sid,
                auth_token,
                from_number,
            } if account_sid.trim().is_empty()
                || auth_token.trim().is_empty()
                || from_number.trim().is_empty()
        ) {
            return Err(anyhow!(
                "TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN and TWILIO_FROM_NUMBER must be set when SMS_PROVIDER=twilio"
            ));
        }

        if matches!(
            &self.sms_credentials,
            SmsProviderCredentials::Orange {
                client_id,
                client_secret,
                ..
            } if client_id.trim().is_empty() || client_secret.trim().is_empty()
        ) {
            return Err(anyhow!(
                "ORANGE_CLIENT_ID and ORANGE_CLIENT_SECRET must be set when SMS_PROVIDER=orange"
            ));
        }

        Ok(())
    }

    /// Check if running in production environment
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Check if running in local environment
    pub fn is_local(&self) -> bool {
        self.environment == Environment::Local
    }

    /// Check if we should use the mock SMS provider
    pub fn use_mock_sms(&self) -> bool {
        self.sms_credentials.is_mock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert!(matches!(LogLevel::from_str("info"), Ok(LogLevel::Info)));
        assert!(matches!(LogLevel::from_str("INFO"), Ok(LogLevel::Info)));
        assert!(matches!(LogLevel::from_str("debug"), Ok(LogLevel::Debug)));
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_environment_from_str() {
        assert!(matches!(
            Environment::from_str("local"),
            Ok(Environment::Local)
        ));
        assert!(matches!(
            Environment::from_str("LOCAL"),
            Ok(Environment::Local)
        ));
        assert!(matches!(
            Environment::from_str("staging"),
            Ok(Environment::Staging)
        ));
        assert!(matches!(
            Environment::from_str("production"),
            Ok(Environment::Production)
        ));
        assert!(Environment::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_level_as_tracing_level() {
        assert_eq!(LogLevel::Trace.as_tracing_level(), tracing::Level::TRACE);
        assert_eq!(LogLevel::Debug.as_tracing_level(), tracing::Level::DEBUG);
        assert_eq!(LogLevel::Info.as_tracing_level(), tracing::Level::INFO);
        assert_eq!(LogLevel::Warn.as_tracing_level(), tracing::Level::WARN);
        assert_eq!(LogLevel::Error.as_tracing_level(), tracing::Level::ERROR);
    }

    #[test]
    fn test_config_helpers() {
        let config = Config {
            database_url: "db".into(),
            server_host: "0.0.0.0".into(),
            server_port: 3000,
            log_level: LogLevel::Info,
            jwt_private_key_pem: "priv".into(),
            jwt_public_key_pem: "pub".into(),
            environment: Environment::Local,
            sms_credentials: SmsProviderCredentials::Vonage {
                api_key: "key".into(),
                api_secret: "secret".into(),
            },
            email_credentials: EmailProviderCredentials::Mock,
            activation_code_pepper: "pepper_longer_than_32_characters_for_test".into(),
            admin_bootstrap_email: Some("admin@iviss.local".into()),
            admin_bootstrap_password: Some("ChangeMe!2025".into()),
            admin_bootstrap_phone: Some("+237600000000".into()),
            admin_bootstrap_username: Some("admin".into()),
        };

        assert!(config.is_local());
        assert!(!config.is_production());
        assert!(config.validate().is_ok());

        let mut prod_config = config.clone();
        prod_config.environment = Environment::Production;
        assert!(prod_config.is_production());
        assert!(!prod_config.is_local());

        // Mock credentials should fail validation in production
        prod_config.sms_credentials = SmsProviderCredentials::Mock;
        assert!(prod_config.validate().is_err());
    }
}
