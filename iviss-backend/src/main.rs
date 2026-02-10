mod config;
mod db;
mod errors;
mod middleware;
mod routes;

use crate::config::Config;
use crate::db::initialize_pool;
use anyhow::Context;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration first (fail-fast if config is invalid)
    let config = Config::from_env().context("Failed to load configuration")?;
    
    // Validate configuration
    config.validate().context("Configuration validation failed")?;

    // Initialize logging based on configuration
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(config.log_level.as_tracing_level())
        .compact()
        .init();

    info!("Starting IVISS Backend...");
    info!("Environment: {:?}", config.environment);
    info!("Log Level: {:?}", config.log_level);

    let pool = initialize_pool(&config.database_url).await?;
    info!("Database connection initialized");

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations completed");

    let app = routes::assembly(pool);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
