pub mod api_doc;
mod config;
mod db;
mod dto;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod app_state;
mod services;
mod queries;

use crate::api_doc::ApiDoc;
use crate::config::Config;
use crate::db::initialize_pool;
use anyhow::Context;
use std::net::SocketAddr;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration first (fail-fast if config is invalid)
    let config = Config::from_env().context("Failed to load configuration")?;

    // Validate configuration
    config
        .validate()
        .context("Configuration validation failed")?;

    // Initialize logging based on configuration
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(config.log_level.as_tracing_level())
        .compact()
        .init();

    info!("Starting IVISS Backend...");
    info!("Environment: {:?}", config.environment);
    info!("Log Level: {:?}", config.log_level);
    let config = Config::from_env()?;

    let pool = initialize_pool(&config.database_url).await?;
    info!("Database connection initialized");

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations completed");

    let app = routes::assembly(pool)
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
