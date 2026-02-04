mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use crate::config::Config;
use crate::db::initialize_pools;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting IVISS Backend...");

    let config = Config::from_env()?;

    let pools = initialize_pools(&config.database_url, &config.external_database_url).await?;
    info!("Database connections initialized");

    let app = routes::assembly(pools);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
