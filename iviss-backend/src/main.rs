use anyhow::Context;
use iviss_backend::api_doc::ApiDoc;
use iviss_backend::app_state::AppState;
use iviss_backend::app_cache::AppCache;
use iviss_backend::config::Config;
use iviss_backend::db::initialize_pool;
use iviss_backend::db::seed_admin::run_bootstrap_seed;
use iviss_backend::routes;
use iviss_backend::services::sms_provider::{MockSmsProvider, SmsProvider, TwilioSmsProvider};
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

    let sms_provider: Arc<dyn SmsProvider> = if !config.use_mock_sms() {
        info!("Using Twilio SMS provider");
        Arc::new(TwilioSmsProvider::new(
            config.twilio_account_sid.clone(),
            config.twilio_auth_token.clone(),
            config.twilio_from_number.clone(),
        ))
    } else {
        info!("Using Mock SMS provider (logs OTP to console)");
        Arc::new(MockSmsProvider)
    };
    let db_pool = initialize_pool(&config.database_url).await?;
    info!("Database connection initialized");

    let cache = Arc::new(AppCache::new());
    info!("App cache initialized");

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Migrations completed");

    info!("Running admin bootstrap seed...");
    run_bootstrap_seed(&db_pool, &config).await;

    let state = AppState::new(db_pool, cache, sms_provider, &config);
    let app = routes::assembly(state)
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
