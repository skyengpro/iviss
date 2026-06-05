use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::{Connection, Executor};
use std::str::FromStr;
use std::time::Duration;

pub type DbPool = PgPool;

async fn ensure_database_exists(database_url: &str) -> Result<()> {
    let options =
        PgConnectOptions::from_str(database_url).context("Failed to parse database URL")?;

    let db_name = options
        .get_database()
        .context("Database name not found in URL")?;

    // Connect to the default 'postgres' database to check/create the target database
    let base_options = options.clone().database("postgres");

    let mut conn = sqlx::postgres::PgConnection::connect_with(&base_options)
        .await
        .context("Failed to connect to 'postgres' database to check target database existence")?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
            .bind(db_name)
            .fetch_one(&mut conn)
            .await
            .context("Failed to query pg_database")?;

    if !exists {
        tracing::info!("Database '{db_name}' does not exist. Creating it...");
        let query = format!("CREATE DATABASE \"{db_name}\"");
        conn.execute(query.as_str())
            .await
            .context(format!("Failed to create database '{db_name}'"))?;
    }

    Ok(())
}

pub async fn initialize_pool(database_url: &str) -> Result<DbPool> {
    let mut retry_count = 0;
    let max_retries = 10;
    let retry_interval = Duration::from_secs(2);

    loop {
        // First, ensure the database exists
        if let Err(e) = ensure_database_exists(database_url).await {
            retry_count += 1;
            if retry_count >= max_retries {
                return Err(e).context("Failed to ensure database exists after multiple attempts");
            }
            tracing::warn!(
                "Failed to ensure database exists (attempt {}/{}). Database might still be starting. Retrying in {:?}... Error: {}",
                retry_count, max_retries, retry_interval, e
            );
            tokio::time::sleep(retry_interval).await;
            continue;
        }

        // Then, connect to the target database pool
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(anyhow::anyhow!(
                        "Failed to connect to database after {max_retries} attempts: {e}"
                    ));
                }
                tracing::warn!(
                    "Failed to connect to database (attempt {}/{}). Retrying in {:?}... Error: {}",
                    retry_count,
                    max_retries,
                    retry_interval,
                    e
                );
                tokio::time::sleep(retry_interval).await;
            }
        }
    }
}
