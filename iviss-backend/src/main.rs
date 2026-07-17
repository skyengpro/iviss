use anyhow::Context;
use iviss_backend::api_doc::ApiDoc;
use iviss_backend::app_cache::AppCache;
use iviss_backend::app_state::AppState;
use iviss_backend::config::Config;
use iviss_backend::db::initialize_pool;
use iviss_backend::db::seed_admin::run_bootstrap_seed;
use iviss_backend::routes;
use iviss_backend::services::email_provider::EmailProvider;
use iviss_backend::services::sms_provider::SmsProvider;
use iviss_backend::services::vehicle_data_cache::{S3VehicleDataCache, VehicleDataCache};
use iviss_backend::telemetry;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().context("Failed to load configuration")?;

    config
        .validate()
        .context("Configuration validation failed")?;

    let telemetry_handle =
        Arc::new(telemetry::init_telemetry(&config.log_level).context("Failed to init telemetry")?);

    info!("Starting IVISS Backend...");
    info!("Environment: {:?}", config.environment);
    info!("Log Level: {:?}", config.log_level);

    let sms_provider: Arc<dyn SmsProvider> = config.sms_credentials.provider();
    let email_provider: Arc<dyn EmailProvider> = config.email_credentials.provider();

    let db_pool = initialize_pool(&config.database_url).await?;
    info!("Database connection initialized");

    let cache = Arc::new(AppCache::new());
    info!("App cache initialized");

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Migrations completed");

    info!("Running admin bootstrap seed...");
    run_bootstrap_seed(&db_pool, &config).await;

    info!("Caching necessary data from database...");
    cache.cache_necessary_data_from_database(&db_pool).await?;

    let s3_data_cache: Option<Arc<dyn VehicleDataCache>> = if config.s3_cache.enabled {
        info!("Initializing S3-compatible vehicle data cache");
        Some(Arc::new(
            S3VehicleDataCache::from_config(&config.s3_cache)
                .await
                .context("Failed to initialize S3-compatible vehicle data cache")?,
        ))
    } else {
        None
    };

    let state = AppState::new(
        db_pool,
        cache,
        sms_provider,
        email_provider,
        &config,
        telemetry_handle.clone(),
        s3_data_cache,
    )
    .context("Failed to initialize application state")?;

    let shared_state = Arc::new(state);

    let mut app = routes::assembly(shared_state.clone());

    if config.environment != iviss_backend::config::Environment::Production {
        app = app.merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));
    }

    let metrics_app = routes::metrics_router(shared_state.clone());

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Listening on {}", addr);

    let metrics_addr: SocketAddr = format!("{}:9091", config.server_host).parse()?;
    info!("Metrics listening on {}", metrics_addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let metrics_listener = tokio::net::TcpListener::bind(metrics_addr).await?;

    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
        info!("Shutdown signal received, draining...");
    };

    // Run both servers concurrently; either shutdown or error stops both.
    tokio::select! {
        r = axum::serve(listener, app).with_graceful_shutdown(shutdown) => r?,
        r = axum::serve(metrics_listener, metrics_app) => r?,
    };

    // Flush telemetry while the Tokio runtime is still active.
    telemetry_handle.shutdown().await;
    info!("Telemetry flushed. Goodbye.");

    Ok(())
}
