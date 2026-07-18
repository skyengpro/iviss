//! iviss-test-api — Mock external vehicle registry API
//!
//! Exposes the same HTTP interface as the real external API so that
//! `VehicleApiService` (in `iviss-backend`) can be pointed at this service
//! via `EXTERNAL_API_BASE_URL` without any code changes.
//!
//! Additionally exposes `GET /batch?prefix=XX` for `s3-cache-sync` to fetch
//! all vehicles sharing a plate prefix in one request.
//!
//! # Configuration (env vars)
//!
//! | Variable              | Default     | Description                       |
//! |-----------------------|-------------|-----------------------------------|
//! | `DATABASE_URL`        | required    | PostgreSQL connection string       |
//! | `TEST_API_USERNAME`   | `test`      | Expected Basic Auth username       |
//! | `TEST_API_PASSWORD`   | `test`      | Expected Basic Auth password       |
//! | `SERVER_PORT`         | `3001`      | Port to listen on                  |
//! | `RUST_LOG`            | `info`      | Log filter                         |
//!
//! # Running
//!
//! ```bash
//! DATABASE_URL=postgres://iviss_user:pass@localhost:5435/iviss_dev \
//! TEST_API_USERNAME=test TEST_API_PASSWORD=test \
//! cargo run
//! ```

mod auth;
mod db;
mod html_builder;
mod routes;

use anyhow::Context;
use auth::ApiCredentials;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tower_http::trace::TraceLayer;

// ── Shared application state ─────────────────────────────────────────────────

/// State available to every request handler.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool:        sqlx::PgPool,
    pub credentials: Arc<ApiCredentials>,
}

/// Allow the Basic Auth extractor to borrow `ApiCredentials` from `AppState`.
impl AsRef<ApiCredentials> for AppState {
    fn as_ref(&self) -> &ApiCredentials {
        // `Arc<T>` derefs to `T`, but in an `AsRef` impl body we must be
        // explicit: `Arc::as_ref()` gives `&T`, not `&Arc<T>`.
        self.credentials.as_ref()
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load .env first so RUST_LOG (and other vars) are set before tracing init.
    dotenvy::dotenv().ok();

    // 2. Initialise tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Starting iviss-test-api mock service");

    // 3. Read configuration from environment
    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL environment variable is required")?;

    let username = env::var("TEST_API_USERNAME").unwrap_or_else(|_| "test".to_string());
    let password = env::var("TEST_API_PASSWORD").unwrap_or_else(|_| "test".to_string());
    let port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .context("SERVER_PORT must be a valid port number")?;

    tracing::info!(port, "Configuration loaded");

    // 4. Connect to PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    tracing::info!("Connected to PostgreSQL");

    // 5. Apply schema DDL and seed data (idempotent)
    db::init(&pool).await?;

    // 6. Build shared state
    let state = AppState {
        pool,
        credentials: Arc::new(ApiCredentials { username, password }),
    };

    // 7. Build Axum router
    let app = Router::new()
        .route("/query",  post(routes::query::query_plate))
        .route("/batch",  get(routes::batch::batch_by_prefix))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 8. Bind and serve
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    tracing::info!(address = %addr, "iviss-test-api listening");

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}

// ── Health endpoint ───────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "OK"
}
