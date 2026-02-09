mod config;
mod db;
mod errors;
mod middleware;
mod routes;

use crate::config::Config;
use crate::db::initialize_pool;
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
