use iviss_backend::api_doc::ApiDoc;
use iviss_backend::app_state::AppState;
use iviss_backend::config::{Config, Environment};
use iviss_backend::db::initialize_pool;
use iviss_backend::db::initialize_redis_pool;
use iviss_backend::services::sms_provider::{MockSmsProvider, SmsProvider, TwilioSmsProvider};
use iviss_backend::routes;

use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;
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

    let sms_provider: Arc<dyn SmsProvider> = match &config.environment {
        Environment::Production => Arc::new(TwilioSmsProvider::new(
            config.twilio_account_sid.clone(),
            config.twilio_auth_token.clone(),
            config.twilio_from_number.clone(),
        )),
        _ => Arc::new(MockSmsProvider),
    };
    let db_pool = initialize_pool(&config.database_url).await?;
    info!("Database connection initialized");

    let redis_pool = initialize_redis_pool(&config.redis_url).await?;
    info!("Redis connection initialized");

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Migrations completed");

    let state = AppState::new(db_pool, redis_pool, sms_provider, config.clone());
    let app = routes::assembly(state)
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
