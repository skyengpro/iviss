use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use hmac::Mac;
/// Shared test helpers for integration tests
/// This module provides common setup functions and utilities used across all integration tests
use iviss_backend::app_cache::AppCache;
use iviss_backend::app_state::AppState;
use iviss_backend::config::{
    Config, EmailProviderCredentials, Environment, LogLevel, SmsProviderCredentials,
};
use iviss_backend::services::email_provider::MockEmailProvider;
use iviss_backend::services::sms_provider::NoopSmsProvider;
use rand::rngs::OsRng;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::RsaPrivateKey;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

const TEST_PEPPER: &str = "test_pepper_for_activation_code_hashing_must_be_32_chars_long";

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

/// Hash password using Argon2 (for test users)
pub fn hash_test_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

/// Generate RSA keypair for JWT signing in tests
pub fn generate_test_rsa_keypair_pem() -> (String, String) {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
    let public_key = private_key.to_public_key();

    let private_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("Failed to encode RSA private key")
        .to_string();
    let public_pem = public_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("Failed to encode RSA public key")
        .to_string();

    (private_pem, public_pem)
}

/// Hash OTP code using the same method as OtpService
pub fn hash_otp_code(pepper: &str, code: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts any key size");
    mac.update(code.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

/// Store OTP directly in Moka cache for testing
pub async fn store_test_otp(
    cache: &AppCache,
    user_id: Uuid,
    code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let code_hash = hash_otp_code(TEST_PEPPER, code);
    cache
        .otp_store
        .insert(
            user_id,
            iviss_backend::app_cache::OtpEntry {
                code_hash,
                attempts: 0,
                expires_at: std::time::Instant::now()
                    + std::time::Duration::from_secs(
                        iviss_backend::services::otp_service::OTP_TTL_SECS,
                    ),
            },
        )
        .await;

    Ok(())
}

/// Setup PostgreSQL container with testcontainers
pub async fn setup_postgres_container() -> (ContainerAsync<Postgres>, String) {
    let pg = Postgres::default().with_host_auth().start().await.unwrap();
    let pg_port = pg.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres@127.0.0.1:{}/postgres", pg_port);
    (pg, db_url)
}

/// Create database pool and run migrations
pub async fn setup_database(db_url: &str) -> PgPool {
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    db
}

/// Create test configuration
pub fn create_test_config(db_url: String) -> Config {
    let (jwt_private_key_pem, jwt_public_key_pem) = generate_test_rsa_keypair_pem();

    Config {
        database_url: db_url,
        server_host: "0.0.0.0".to_string(),
        server_port: 0,
        log_level: LogLevel::Info,
        jwt_private_key_pem,
        jwt_public_key_pem,
        environment: Environment::Local,
        sms_credentials: SmsProviderCredentials::Mock,
        email_credentials: EmailProviderCredentials::Mock,
        activation_code_pepper: TEST_PEPPER.to_string(),
        admin_bootstrap_email: Some("admin@example.com".to_string()),
        admin_bootstrap_password: Some("password".to_string()),
        admin_bootstrap_phone: Some("1234567890".to_string()),
        admin_bootstrap_username: Some("admin".to_string()),
    }
}

/// Create test AppState
pub fn create_test_app_state(db: PgPool, cache: Arc<AppCache>, config: &Config) -> AppState {
    AppState::new(
        db,
        cache,
        Arc::new(NoopSmsProvider),
        Arc::new(MockEmailProvider),
        config,
    )
}

/// Insert test organization and return its ID
pub async fn insert_test_organization(db: &PgPool, name: &str, org_type: &str) -> Uuid {
    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, type, start_work_time, end_work_time) 
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(org_id)
    .bind(name)
    .bind(org_type)
    .bind(360i32) // 6:00 AM
    .bind(1080i32) // 6:00 PM
    .execute(db)
    .await
    .expect("Failed to insert test organization");

    org_id
}

/// Insert test agent user and return user ID
pub async fn insert_test_agent(
    db: &PgPool,
    org_id: Uuid,
    badge_id: &str,
    phone: &str,
    status: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, role, badge_id, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4::user_role, $5, $6, $7, $8::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind(format!("agent_{}", badge_id))
    .bind("agent")
    .bind(badge_id)
    .bind(format!("Agent {}", badge_id))
    .bind(phone)
    .bind(status)
    .execute(db)
    .await
    .expect("Failed to insert test agent");

    user_id
}

/// Insert test admin user and return user ID
pub async fn insert_test_admin(
    db: &PgPool,
    org_id: Uuid,
    email: &str,
    password_hash: &str,
    role: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, organization_id, username, email, password_hash, role, full_name, phone_number, status)
        VALUES ($1, $2, $3, $4, $5, $6::user_role, $7, $8, 'ACTIVE'::user_status)
        "#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind(email.split('@').next().unwrap())
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .bind(format!("Admin {}", email))
    .bind("+237600000000")
    .execute(db)
    .await
    .expect("Failed to insert test admin");

    user_id
}

/// Insert test device and return device ID
pub async fn insert_test_device(db: &PgPool, user_id: Uuid, status: &str) -> Uuid {
    let device_id = Uuid::new_v4();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let shift_start = now;
    let shift_end = now + 3600;

    // Generate unique public key for each device
    let public_key = format!("test_public_key_{}", device_id);

    sqlx::query(
        r#"
        INSERT INTO devices (id, user_id, public_key, status, metadata)
        VALUES ($1, $2, $3, $4::device_status, jsonb_build_object('shift_start', $5, 'shift_end', $6))
        "#,
    )
    .bind(device_id)
    .bind(user_id)
    .bind(public_key)
    .bind(status)
    .bind(shift_start as i64)
    .bind(shift_end as i64)
    .execute(db)
    .await
    .expect("Failed to insert test device");

    device_id
}

/// Generate JWT token for testing with proper device_id
/// For admin/manager/org_admin roles, use Uuid::nil() as device_id
/// For agent roles, use the actual device_id
pub async fn generate_test_jwt_token(
    config: &Config,
    user_id: Uuid,
    device_id: Uuid,
    role: &str,
) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: Uuid,
        device_id: Uuid,
        role: String,
        shift_start: usize,
        shift_end: usize,
        exp: usize,
        jti: Uuid,
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let shift_start = now as usize;

    // Admins get 24 hour sessions, agents get 8 hour sessions
    let shift_duration =
        if role == "admin" || role == "manager" || role == "org_admin" || role == "super_admin" {
            24 * 60 * 60 // 24 hours
        } else {
            8 * 60 * 60 // 8 hours
        };

    let shift_end = (now + shift_duration) as usize;
    let expiration = shift_end; // Token expires at shift end

    // Admins use Uuid::nil() as device_id, agents use actual device_id
    let token_device_id = if role == "admin" || role == "manager" || role == "org_admin" {
        Uuid::nil()
    } else {
        device_id
    };

    let claims = Claims {
        sub: user_id,
        device_id: token_device_id,
        role: role.to_string(),
        shift_start,
        shift_end,
        exp: expiration,
        jti: Uuid::new_v4(),
    };

    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.typ = Some("JWT".to_string());

    let key = EncodingKey::from_rsa_pem(config.jwt_private_key_pem.as_bytes())
        .expect("Failed to create encoding key");

    encode(&header, &claims, &key).expect("Failed to encode JWT")
}

/// Complete test infrastructure setup
/// Returns (app, db, org_id, user_id, device_id, container, cache, config)
pub async fn setup_complete_test_infrastructure() -> (
    axum::Router,
    PgPool,
    Uuid,
    Uuid,
    Uuid,
    ContainerAsync<Postgres>,
    Arc<AppCache>,
    Config,
) {
    let (pg, db_url) = setup_postgres_container().await;
    let db = setup_database(&db_url).await;
    let cache = Arc::new(AppCache::new());
    let config = create_test_config(db_url);

    // Create test organization
    let org_id = insert_test_organization(&db, "Test Org", "police").await;

    // Create test agent
    let user_id = insert_test_agent(&db, org_id, "AGENT-001", "+237600000001", "ACTIVE").await;

    // Create test device
    let device_id = insert_test_device(&db, user_id, "ACTIVE").await;

    // Create app state and router
    let state = create_test_app_state(db.clone(), cache.clone(), &config);
    let app = iviss_backend::routes::assembly(state);

    (app, db, org_id, user_id, device_id, pg, cache, config)
}
