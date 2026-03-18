use crate::config::Config;
use crate::utils::password::hash_password;
use sqlx::PgPool;

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
            tracing::debug!(
                "Bootstrap: ADMIN_BOOTSTRAP_* env vars not set — skipping seed"
            );
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Bootstrap: seed failed — check ADMIN_BOOTSTRAP_* env vars"
            );
        }
    }
}

enum BootstrapResult {
    Created(String),
    AlreadyExists,
    Skipped,
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
            status
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
            'ACTIVE'::user_status
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