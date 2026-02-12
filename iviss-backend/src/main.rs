pub mod api_doc;
mod config;
mod db;
mod dto;
mod errors;
mod handlers;
mod middleware;
mod routes;
use crate::api_doc::ApiDoc;
use crate::config::Config;
// use crate::db::initialize_pool;
use std::net::SocketAddr;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting IVISS Backend...");

    let config = Config::from_env()?;

    // let pool = initialize_pool(&config.database_url).await?;
    info!("Database connection initialized");

    let app = routes::assembly(/* pool */)
        .merge(SwaggerUi::new("/docs").url("/api-doc.json", ApiDoc::openapi()));

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
