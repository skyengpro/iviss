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
    pub server_host: String,
    pub server_port: u16,
    pub log_level: LogLevel,
    #[allow(dead_code)]
    pub jwt_secret: String,
    pub environment: Environment,
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

        // Load and validate JWT_SECRET (critical)
        let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET must be set")?;

        if jwt_secret.trim().is_empty() {
            return Err(anyhow!("JWT_SECRET cannot be empty"));
        }

        // Enforce minimum length for JWT_SECRET for security
        if jwt_secret.len() < 32 {
            return Err(anyhow!(
                "JWT_SECRET must be at least 32 characters long for security. Current length: {}",
                jwt_secret.len()
            ));
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

        Ok(Self {
            database_url,
            server_host,
            server_port,
            log_level,
            jwt_secret,
            environment,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Additional validation can be added here
        if self.server_port == 0 {
            return Err(anyhow!("SERVER_PORT cannot be 0"));
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
            Environment::from_str("production"),
            Ok(Environment::Production)
        ));
        assert!(Environment::from_str("invalid").is_err());
    }
}
