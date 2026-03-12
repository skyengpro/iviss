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
                "Invalid ENVIRONMENT value: '{}'. Must be one of: local, staging, production",
                s
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
                "Invalid LOG_LEVEL value: '{}'. Must be one of: trace, debug, info, warn, error",
                s
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
#[allow(dead_code)] // jwt_secret and helper methods will be used in future JWT implementation
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub log_level: LogLevel,
    #[allow(dead_code)]
    pub jwt_secret: String,
    pub jwt_private_key_pem: String,
    pub jwt_public_key_pem: String,
    pub environment: Environment,
    // SMS
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_from_number: String,
    pub activation_code_pepper: String,
    pub shift_start_hour: u32,
    pub shift_end_hour: u32,
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
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        if redis_url.trim().is_empty() {
            return Err(anyhow!("REDIS_URL cannot be empty"));
        }
        // Load and validate JWT_PRIVATE_KEY_PEM (critical)
        let jwt_secret =
            env::var("JWT_PRIVATE_KEY_PEM").context("JWT_PRIVATE_KEY_PEM must be set")?;

        if jwt_secret.trim().is_empty() {
            return Err(anyhow!("JWT_PRIVATE_KEY_PEM cannot be empty"));
        }

        // Enforce minimum length for JWT_PRIVATE_KEY_PEM for security
        if jwt_secret.len() < 32 {
            return Err(anyhow!(
                "JWT_PRIVATE_KEY_PEM must be at least 32 characters long for security. Current length: {}",
                jwt_secret.len()
            ));
        }

        let jwt_private_key_pem =
            env::var("JWT_PRIVATE_KEY_PEM").context("JWT_PRIVATE_KEY_PEM must be set")?
            .replace("\\n", "\n");
        if jwt_private_key_pem.trim().is_empty() {
            return Err(anyhow!("JWT_PRIVATE_KEY_PEM cannot be empty"));
        }

        let jwt_public_key_pem =
            env::var("JWT_PUBLIC_KEY_PEM").context("JWT_PUBLIC_KEY_PEM must be set")?
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

        // Twilio — requis uniquement en production
        let twilio_account_sid =
            env::var("TWILIO_ACCOUNT_SID").unwrap_or_else(|_| "mock".to_string());
        let twilio_auth_token =
            env::var("TWILIO_AUTH_TOKEN").unwrap_or_else(|_| "mock".to_string());
        let twilio_from_number =
            env::var("TWILIO_FROM_NUMBER").unwrap_or_else(|_| "mock".to_string());

        let activation_code_pepper =
            env::var("ACTIVATION_CODE_PEPPER").context("ACTIVATION_CODE_PEPPER must be set")?;

        if activation_code_pepper.len() < 32 {
            return Err(anyhow!(
                "ACTIVATION_CODE_PEPPER must be at least 32 characters"
            ));
        }
        let shift_start_hour = env::var("SHIFT_START_HOUR")
            .unwrap_or_else(|_| "6".to_string())
            .parse::<u32>()
            .context("SHIFT_START_HOUR must be a valid hour (0-23)")?;

        let shift_end_hour = env::var("SHIFT_END_HOUR")
            .unwrap_or_else(|_| "18".to_string())
            .parse::<u32>()
            .context("SHIFT_END_HOUR must be a valid hour (0-23)")?;

        if shift_start_hour > 23 || shift_end_hour > 23 {
            return Err(anyhow!(
                "SHIFT_START_HOUR and SHIFT_END_HOUR must be between 0 and 23"
            ));
        }
        if shift_start_hour >= shift_end_hour {
            return Err(anyhow!(
                "SHIFT_START_HOUR ({}) must be less than SHIFT_END_HOUR ({})",
                shift_start_hour,
                shift_end_hour
            ));
        }
        Ok(Self {
            database_url,
            redis_url,
            server_host,
            server_port,
            log_level,
            jwt_secret,
            jwt_private_key_pem,
            jwt_public_key_pem,
            environment,
            twilio_account_sid,
            twilio_auth_token,
            twilio_from_number,
            activation_code_pepper,
            shift_start_hour,
            shift_end_hour,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Additional validation can be added here
        if self.server_port == 0 {
            // return Err(anyhow!("SERVER_PORT cannot be 0"));
        }
        // Validate Twilio config
        if self.environment == Environment::Production
            && (self.twilio_account_sid == "mock"
                || self.twilio_auth_token == "mock"
                || self.twilio_from_number == "mock")
        {
            return Err(anyhow!("Twilio credentials must be set in production"));
        }

        Ok(())
    }

    /// Check if running in production environment
    #[allow(dead_code)]
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Check if running in local environment
    #[allow(dead_code)]
    pub fn is_local(&self) -> bool {
        self.environment == Environment::Local
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
            redis_url: "redis".into(),
            server_host: "0.0.0.0".into(),
            server_port: 3000,
            log_level: LogLevel::Info,
            jwt_secret: "secret_longer_than_32_characters_for_test".into(),
            jwt_private_key_pem: "priv".into(),
            jwt_public_key_pem: "pub".into(),
            environment: Environment::Local,
            twilio_account_sid: "sid".into(),
            twilio_auth_token: "token".into(),
            twilio_from_number: "num".into(),
            activation_code_pepper: "pepper_longer_than_32_characters_for_test".into(),
            shift_start_hour: 8,
            shift_end_hour: 18,
        };

        assert!(config.is_local());
        assert!(!config.is_production());
        assert!(config.validate().is_ok());

        let mut prod_config = config.clone();
        prod_config.environment = Environment::Production;
        assert!(prod_config.is_production());
        assert!(!prod_config.is_local());

        // Twilio check in production
        prod_config.twilio_account_sid = "mock".into();
        assert!(prod_config.validate().is_err());
    }
}
