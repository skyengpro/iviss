use crate::config::Config;
use crate::utils::password::hash_password;
use sqlx::PgPool;

enum BootstrapResult {
    Created(String),
    AlreadyExists,
    Skipped,
}
/// Run the admin bootstrap seed at application startup.
///
/// Idempotent — skips silently if an admin already exists.
/// Non-fatal — logs a warning on error, never crashes the app.
pub async fn run_bootstrap_seed(pool: &PgPool, config: &Config) {
    match try_bootstrap(pool, config).await {
        Ok(BootstrapResult::Created(email)) => {
            tracing::info!(
                email = %email,
                "Bootstrap: first admin created — change the default password immediately"
            );
        }
        Ok(BootstrapResult::AlreadyExists) => {
            tracing::debug!("Bootstrap: admin already exists — skipping seed");
        }
        Ok(BootstrapResult::Skipped) => {
            tracing::debug!("Bootstrap: ADMIN_BOOTSTRAP_* env vars not set — skipping seed");
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Bootstrap: seed failed — check ADMIN_BOOTSTRAP_* env vars"
            );
        }
    }
}

async fn try_bootstrap(pool: &PgPool, config: &Config) -> anyhow::Result<BootstrapResult> {
    // Skip silently if any env var is missing
    let (email, password, phone, username) = match (
        &config.admin_bootstrap_email,
        &config.admin_bootstrap_password,
        &config.admin_bootstrap_phone,
        &config.admin_bootstrap_username,
    ) {
        (Some(e), Some(p), Some(ph), Some(u)) => (e.clone(), p.clone(), ph.clone(), u.clone()),
        _ => return Ok(BootstrapResult::Skipped),
    };

    // Idempotency check
    let admin_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE role = 'admin' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    if admin_count > 0 {
        return Ok(BootstrapResult::AlreadyExists);
    }

    // Hash password with argon2id
    let password_hash = hash_password(&password).await?;

    sqlx::query(
        r#"
        INSERT INTO users (
            id,
            organization_id,
            username,
            email,
            password_hash,
            role,
            full_name,
            phone_number,
            status,
            must_change_password
        )
        VALUES (
            uuid_generate_v4(),
            NULL,
            $1,
            $2,
            $3,
            'admin'::user_role,
            'System Administrator',
            $4,
            'ACTIVE'::user_status,
            TRUE
        )
        ON CONFLICT (email) DO NOTHING
        "#,
    )
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .bind(&phone)
    .execute(pool)
    .await?;

    Ok(BootstrapResult::Created(email))
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    fn create_test_config(
        db_url: &str,
        email: &str,
        password: &str,
        phone: &str,
        username: &str,
    ) -> Config {
        Config {
            database_url: db_url.to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 8080,
            log_level: crate::config::LogLevel::Info,
            jwt_private_key_pem: "test_key".to_string(),
            jwt_public_key_pem: "test_pub".to_string(),
            environment: crate::config::Environment::Local,
            sms_credentials: crate::services::sms_provider::SmsProviderCredentials::Mock,
            email_credentials: crate::services::email_provider::EmailProviderCredentials::Mock,
            otp_via_email: false,
            activation_code_pepper: "pepper_longer_than_32_characters_for_test".to_string(),
            admin_bootstrap_email: Some(email.to_string()),
            admin_bootstrap_password: Some(password.to_string()),
            admin_bootstrap_phone: Some(phone.to_string()),
            admin_bootstrap_username: Some(username.to_string()),
            vehicle_api_credentials: crate::config::mock_vehicle_api_credentials(),
        }
    }

    async fn setup_test_db() -> (sqlx::PgPool, testcontainers::ContainerAsync<Postgres>) {
        let postgres = Postgres::default().with_host_auth().start().await.unwrap();
        let host = postgres.get_host().await.unwrap();
        let port = postgres.get_host_port_ipv4(5432).await.unwrap();
        let db_url = format!("postgres://postgres@{host}:{port}/postgres");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        (pool, postgres)
    }

    #[tokio::test]
    async fn test_bootstrap_admin_creates_new_admin() {
        let (pool, _pg) = setup_test_db().await;
        let db_url = "postgres://test".to_string();
        let email = "admin@test.com";
        let config = create_test_config(&db_url, email, "password123", "+1234567890", "admin");

        let result = try_bootstrap(&pool, &config).await.unwrap();

        assert!(matches!(result, BootstrapResult::Created(_)));
        if let BootstrapResult::Created(result_email) = result {
            assert_eq!(result_email, email);
        }

        // Verify user was created with correct role and status
        let user: (String, String) =
            sqlx::query_as("SELECT role::text, status::text FROM users WHERE email = $1")
                .bind(email)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(user.0, "admin");
        assert_eq!(user.1, "ACTIVE");
    }

    #[tokio::test]
    async fn test_bootstrap_admin_skips_when_already_exists() {
        let (pool, _pg) = setup_test_db().await;
        let db_url = "postgres://test".to_string();
        let email = "admin@test.com";
        let config = create_test_config(&db_url, email, "password123", "+1234567890", "admin");

        // First call - creates the admin
        let result1 = try_bootstrap(&pool, &config).await.unwrap();
        assert!(matches!(result1, BootstrapResult::Created(_)));

        // Second call - should return AlreadyExists
        let result2 = try_bootstrap(&pool, &config).await.unwrap();
        assert!(matches!(result2, BootstrapResult::AlreadyExists));

        // Verify only one user exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_bootstrap_admin_password_is_hashed() {
        let (pool, _pg) = setup_test_db().await;
        let db_url = "postgres://test".to_string();
        let email = "admin@test.com";
        let plain_password = "password123";
        let config = create_test_config(&db_url, email, plain_password, "+1234567890", "admin");

        try_bootstrap(&pool, &config).await.unwrap();

        // Verify password is hashed (not stored in plain text)
        let password_hash: String =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE email = $1")
                .bind(email)
                .fetch_one(&pool)
                .await
                .unwrap();

        // Password should not be equal to plain text
        assert_ne!(password_hash, plain_password);
        // Should be a valid argon2 hash format
        assert!(password_hash.starts_with("$argon2"));
    }
}
