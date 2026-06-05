use sqlx::PgPool;

/// Run the development seed data at application startup.
///
/// Idempotent — uses ON CONFLICT DO NOTHING so it's safe to run on every startup.
/// Non-fatal — logs a warning on error, never crashes the app.
/// Only runs when SEED_DATA=true is set in the environment.
pub async fn run_seed_data(pool: &PgPool) {
    if std::env::var("SEED_DATA").as_deref() != Ok("true") {
        tracing::debug!("Seed data: SEED_DATA env var not set to 'true' — skipping");
        return;
    }

    match try_seed(pool).await {
        Ok(()) => tracing::info!("Seed data: populated successfully"),
        Err(e) => tracing::warn!(error = %e, "Seed data: failed to populate"),
    }
}

async fn try_seed(pool: &PgPool) -> anyhow::Result<()> {
    let sql = include_str!("../../seeds/seed_data.sql");
    sqlx::raw_sql(sql).execute(pool).await?;
    Ok(())
}
