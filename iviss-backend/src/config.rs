pub use crate::external_services::vehicle_client::{
    ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials,
};
pub use crate::s3_cache_layer::S3CacheConfig;
pub use crate::services::notifications::email_provider::EmailProviderCredentials;
pub use crate::services::notifications::sms_provider::SmsProviderCredentials;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::env;
use std::str::FromStr;

const DEFAULT_LOCAL_ALLOWED_ORIGINS: &[&str] = &[
    "http://localhost:8080",
    "http://127.0.0.1:8080",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
];

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
    pub cors_allowed_origins: Vec<String>,
    // SMS
    pub sms_credentials: SmsProviderCredentials,
    // Email
    pub email_credentials: EmailProviderCredentials,
    pub otp_via_email: bool,
    pub activation_code_pepper: String,
    // Bootstrap admin — used only at first startup if no admin exists
    pub admin_bootstrap_email: Option<String>,
    pub admin_bootstrap_password: Option<String>,
    pub admin_bootstrap_phone: Option<String>,
    pub admin_bootstrap_username: Option<String>,
    // Vehicle API
    pub vehicle_api_credentials: VehicleApiCredentials,
    // Kill switch: when false, no outbound calls to the vehicle API partner service are made
    pub enable_vehicle_api: bool,
    // S3-compatible vehicle data cache
    pub s3_cache: S3CacheConfig,
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

        let cors_allowed_origins = Self::get_allowed_origins(&environment)
            .context("Failed to parse CORS_ALLOWED_ORIGINS")?;

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

        // Vehicle API credentials
        let vehicle_api_credentials = Self::get_vehicle_api_credentials()
            .context("Failed to configure Vehicle API credentials")?;

        // Vehicle API kill switch (default: enabled)
        let enable_vehicle_api = Self::parse_bool_env("ENABLE_VEHICLE_API", true);

        // OTP delivery via email toggle (default: false)
        let otp_via_email = env::var("OTP_VIA_EMAIL")
            .ok()
            .map(|v| {
                let v = v.trim().to_lowercase();
                matches!(v.as_str(), "1" | "true" | "yes")
            })
            .unwrap_or(false);

        let s3_cache = Self::get_s3_cache_config();

        Ok(Self {
            database_url,
            server_host,
            server_port,
            log_level,
            jwt_private_key_pem,
            jwt_public_key_pem,
            environment,
            cors_allowed_origins,
            sms_credentials,
            email_credentials,
            otp_via_email,
            activation_code_pepper,
            admin_bootstrap_email,
            admin_bootstrap_password,
            admin_bootstrap_phone,
            admin_bootstrap_username,
            vehicle_api_credentials,
            enable_vehicle_api,
            s3_cache,
        })
    }

    fn parse_bool_env(name: &str, default: bool) -> bool {
        env::var(name)
            .ok()
            .map(|v| {
                let v = v.trim().to_lowercase();
                matches!(v.as_str(), "1" | "true" | "yes" | "on")
            })
            .unwrap_or(default)
    }

    fn get_allowed_origins(environment: &Environment) -> Result<Vec<String>> {
        match env::var("CORS_ALLOWED_ORIGINS") {
            Ok(raw) if !raw.trim().is_empty() => Self::parse_allowed_origins(&raw),
            _ if *environment == Environment::Local => Ok(DEFAULT_LOCAL_ALLOWED_ORIGINS
                .iter()
                .map(|origin| (*origin).to_string())
                .collect()),
            _ => Err(anyhow!(
                "CORS_ALLOWED_ORIGINS must be set when ENVIRONMENT is staging or production"
            )),
        }
    }

    fn parse_allowed_origins(raw: &str) -> Result<Vec<String>> {
        let mut origins = Vec::new();

        for origin in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            Self::validate_origin(origin)?;
            origins.push(origin.to_string());
        }

        if origins.is_empty() {
            return Err(anyhow!("CORS_ALLOWED_ORIGINS cannot be empty"));
        }

        Ok(origins)
    }

    fn validate_origin(origin: &str) -> Result<()> {
        if origin == "*" {
            return Err(anyhow!(
                "CORS_ALLOWED_ORIGINS must not contain wildcard '*'"
            ));
        }

        let authority = origin
            .strip_prefix("https://")
            .or_else(|| origin.strip_prefix("http://"))
            .ok_or_else(|| {
                anyhow!(
                    "Invalid origin '{origin}': expected an absolute http:// or https:// origin"
                )
            })?;

        if authority.is_empty() {
            return Err(anyhow!("Invalid origin '{origin}': missing host"));
        }

        if authority.contains(['/', '?', '#']) {
            return Err(anyhow!(
                "Invalid origin '{origin}': origins must not include path, query, fragment, or trailing slash"
            ));
        }

        if origin.bytes().any(|b| b <= 0x20 || b >= 0x7f) {
            return Err(anyhow!(
                "Invalid origin '{origin}': only visible ASCII characters are allowed"
            ));
        }

        Ok(())
    }

    fn get_s3_cache_config() -> S3CacheConfig {
        let enabled = Self::parse_bool_env("S3_CACHE_ENABLED", false);
        let bucket = env::var("S3_CACHE_BUCKET")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let region = env::var("S3_CACHE_REGION")
            .or_else(|_| env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "eu-west-1".to_string());
        let endpoint_url = env::var("S3_CACHE_ENDPOINT_URL")
            .or_else(|_| env::var("AWS_ENDPOINT_URL"))
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let force_path_style =
            Self::parse_bool_env("S3_CACHE_FORCE_PATH_STYLE", endpoint_url.is_some());

        // SSE-KMS: optional KMS key ARN for server-side encryption.
        let kms_key_id = env::var("S3_CACHE_KMS_KEY_ID")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());

        // Client-side AES-256-GCM: optional base64-encoded 32-byte key.
        let encryption_key = env::var("S3_CACHE_ENCRYPTION_KEY")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .map(|b64| {
                use base64::Engine;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(&b64)
                    .expect("S3_CACHE_ENCRYPTION_KEY is not valid base64");
                let key: [u8; 32] = bytes
                    .try_into()
                    .expect("S3_CACHE_ENCRYPTION_KEY must decode to exactly 32 bytes");
                key
            });

        S3CacheConfig {
            enabled,
            bucket,
            region,
            endpoint_url,
            force_path_style,
            kms_key_id,
            encryption_key,
        }
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
            "mock" | "none" => Ok(SmsProviderCredentials::Mock),
            other => Err(anyhow!(
                "Invalid SMS_PROVIDER value: '{other}'. Must be one of: vonage, twilio, orange, mock"
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

    fn get_vehicle_api_credentials() -> Result<VehicleApiCredentials> {
        let base_url =
            env::var("EXTERNAL_API_BASE_URL").context("EXTERNAL_API_BASE_URL must be set")?;
        let username =
            env::var("EXTERNAL_API_USERNAME").context("EXTERNAL_API_USERNAME must be set")?;
        let password =
            env::var("EXTERNAL_API_PASSWORD").context("EXTERNAL_API_PASSWORD must be set")?;
        let lock_ndia =
            env::var("EXTERNAL_API_LOCK_NDIA").context("EXTERNAL_API_LOCK_NDIA must be set")?;
        let kindia = env::var("EXTERNAL_API_KINDIA").context("EXTERNAL_API_KINDIA must be set")?;
        let user = env::var("EXTERNAL_API_USER").context("EXTERNAL_API_USER must be set")?;
        let client = env::var("EXTERNAL_API_CLIENT").context("EXTERNAL_API_CLIENT must be set")?;
        let ctr = env::var("EXTERNAL_API_CTR").context("EXTERNAL_API_CTR must be set")?;
        let tls_cert_b64 = env::var("EXTERNAL_API_TLS_CERT_B64")
            .context("EXTERNAL_API_TLS_CERT_B64 must be set")?;

        Ok(VehicleApiCredentials {
            base_url,
            user_auth: ApiUserAuth { username, password },
            header_parms: ExternalApiHeaderParms {
                user,
                lock_ndia,
                kindia,
                client,
                ctr,
            },
            tls_cert_b64,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.cors_allowed_origins.is_empty() {
            return Err(anyhow!("CORS_ALLOWED_ORIGINS cannot be empty"));
        }

        for origin in &self.cors_allowed_origins {
            Self::validate_origin(origin)?;
        }

        // Validate SMS provider config in production
        if self.environment == Environment::Production {
            // Mock SMS provider is not allowed in production
            if matches!(&self.sms_credentials, SmsProviderCredentials::Mock) {
                return Err(anyhow!(
                    "Mock SMS provider is not allowed in production environment"
                ));
            }
            // Validate Orange credentials if using Orange provider
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

        if self.s3_cache.enabled && self.s3_cache.bucket.is_none() {
            return Err(anyhow!(
                "S3_CACHE_BUCKET must be set when S3_CACHE_ENABLED=true"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
pub fn mock_vehicle_api_credentials() -> VehicleApiCredentials {
    VehicleApiCredentials {
        base_url: "https://vehicle-api.test".into(),
        user_auth: ApiUserAuth {
            username: "test_username".into(),
            password: "test_password".into(),
        },
        header_parms: ExternalApiHeaderParms {
            user: "test_user".into(),
            lock_ndia: "test_lock_ndia".into(),
            kindia: "test_kindia".into(),
            client: "test_client".into(),
            ctr: "test_ctr".into(),
        },
        tls_cert_b64: "TiBDRVJUSUZJQ0FURS0tLS0tCk1JSUZzRENDQTVp".into(),
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
    fn test_parse_allowed_origins_accepts_explicit_origin_list() {
        let origins =
            Config::parse_allowed_origins("https://app.iviss.example,http://localhost:8080")
                .unwrap();

        assert_eq!(
            origins,
            vec![
                "https://app.iviss.example".to_string(),
                "http://localhost:8080".to_string()
            ]
        );
    }

    #[test]
    fn test_parse_allowed_origins_rejects_wildcard() {
        let err = Config::parse_allowed_origins("*").unwrap_err();

        assert!(err.to_string().contains("wildcard"));
    }

    #[test]
    fn test_parse_allowed_origins_rejects_path_or_trailing_slash() {
        let err = Config::parse_allowed_origins("https://app.iviss.example/").unwrap_err();

        assert!(err.to_string().contains("must not include path"));
    }

    #[test]
    fn test_staging_and_production_require_allowed_origins() {
        std::env::remove_var("CORS_ALLOWED_ORIGINS");

        assert!(Config::get_allowed_origins(&Environment::Local).is_ok());
        assert!(Config::get_allowed_origins(&Environment::Staging).is_err());
        assert!(Config::get_allowed_origins(&Environment::Production).is_err());
    }

    #[test]
    fn test_validate_rejects_empty_allowed_origins() {
        let config = Config {
            database_url: "db".into(),
            server_host: "0.0.0.0".into(),
            server_port: 3000,
            log_level: LogLevel::Info,
            jwt_private_key_pem: "priv".into(),
            jwt_public_key_pem: "pub".into(),
            environment: Environment::Local,
            cors_allowed_origins: vec![],
            sms_credentials: SmsProviderCredentials::Mock,
            email_credentials: EmailProviderCredentials::Mock,
            otp_via_email: false,
            activation_code_pepper: "pepper_longer_than_32_characters_for_test".into(),
            admin_bootstrap_email: None,
            admin_bootstrap_password: None,
            admin_bootstrap_phone: None,
            admin_bootstrap_username: None,
            vehicle_api_credentials: mock_vehicle_api_credentials(),
            enable_vehicle_api: true,
            s3_cache: S3CacheConfig::default(),
        };

        assert!(config.validate().is_err());
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
            cors_allowed_origins: vec!["http://localhost:8080".into()],
            sms_credentials: SmsProviderCredentials::Vonage {
                api_key: "key".into(),
                api_secret: "secret".into(),
            },
            email_credentials: EmailProviderCredentials::Mock,
            otp_via_email: false,
            activation_code_pepper: "pepper_longer_than_32_characters_for_test".into(),
            admin_bootstrap_email: Some("admin@iviss.local".into()),
            admin_bootstrap_password: Some("ChangeMe!2025".into()),
            admin_bootstrap_phone: Some("+237600000000".into()),
            admin_bootstrap_username: Some("admin".into()),
            vehicle_api_credentials: mock_vehicle_api_credentials(),
            enable_vehicle_api: true,
            s3_cache: S3CacheConfig::default(),
        };
        assert!(config.validate().is_ok());

        let mut prod_config = config.clone();
        prod_config.environment = Environment::Production;

        // Mock credentials should fail validation in production
        prod_config.sms_credentials = SmsProviderCredentials::Mock;
        assert!(prod_config.validate().is_err());
    }

    #[test]
    fn test_s3_cache_requires_bucket_when_enabled() {
        let mut config = Config {
            database_url: "db".into(),
            server_host: "0.0.0.0".into(),
            server_port: 3000,
            log_level: LogLevel::Info,
            jwt_private_key_pem: "priv".into(),
            jwt_public_key_pem: "pub".into(),
            environment: Environment::Local,
            cors_allowed_origins: vec!["http://localhost:8080".into()],
            sms_credentials: SmsProviderCredentials::Mock,
            email_credentials: EmailProviderCredentials::Mock,
            otp_via_email: false,
            activation_code_pepper: "pepper_longer_than_32_characters_for_test".into(),
            admin_bootstrap_email: None,
            admin_bootstrap_password: None,
            admin_bootstrap_phone: None,
            admin_bootstrap_username: None,
            vehicle_api_credentials: mock_vehicle_api_credentials(),
            enable_vehicle_api: true,
            s3_cache: S3CacheConfig {
                enabled: true,
                bucket: None,
                region: "us-east-1".into(),
                endpoint_url: None,
                force_path_style: false,
                kms_key_id: None,
                encryption_key: None,
            },
        };

        assert!(config.validate().is_err());

        config.s3_cache.bucket = Some("iviss-vehicle-cache".into());
        assert!(config.validate().is_ok());
    }
}
