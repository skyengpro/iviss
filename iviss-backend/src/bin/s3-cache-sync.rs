//! S3 Cache Sync Service
//!
//! Periodically fetches vehicle data from the external API and populates
//! the S3 cache layer. Designed to run as a long-lived service with a
//! tokio-cron-scheduler (1x/day).
//!
//! Build: cargo build --bin s3-cache-sync --no-default-features

use iviss_backend::s3_cache_layer::{self, S3CacheConfig};
use iviss_backend::vehicle_client::{
    ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials, VehicleApiService,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Init tracing (minimal, stdout only — no OpenTelemetry)
    tracing_subscriber::fmt::init();
    tracing::info!("Starting S3 Cache Sync Service...");

    dotenvy::dotenv().ok();

    // 2. Load VehicleApiCredentials from env vars
    let base_url = env::var("EXTERNAL_API_BASE_URL")
        .unwrap_or_else(|_| "https://test-api.iviss.gov".to_string());
    let username = env::var("EXTERNAL_API_USERNAME").unwrap_or_default();
    let password = env::var("EXTERNAL_API_PASSWORD").unwrap_or_default();
    
    let header_user = env::var("EXTERNAL_API_HEADER_USER").unwrap_or_default();
    let header_lock_ndia = env::var("EXTERNAL_API_HEADER_LOCK_NDIA").unwrap_or_default();
    let header_kindia = env::var("EXTERNAL_API_HEADER_KINDIA").unwrap_or_default();
    let header_client = env::var("EXTERNAL_API_HEADER_CLIENT").unwrap_or_default();
    let header_ctr = env::var("EXTERNAL_API_HEADER_CTR").unwrap_or_default();
    let tls_cert_b64 = env::var("EXTERNAL_API_TLS_CERT_B64").unwrap_or_default();

    let api_credentials = VehicleApiCredentials {
        base_url,
        user_auth: ApiUserAuth { username, password },
        header_parms: ExternalApiHeaderParms {
            user: header_user,
            lock_ndia: header_lock_ndia,
            kindia: header_kindia,
            client: header_client,
            ctr: header_ctr,
        },
        tls_cert_b64,
    };

    // 3. Build VehicleApiService (shared module)
    let _vehicle_api_svc = VehicleApiService::new(api_credentials)?;

    // 4. Load S3CacheConfig from env vars
    let bucket = env::var("S3_CACHE_BUCKET").ok();
    let region = env::var("S3_CACHE_REGION").unwrap_or_else(|_| "eu-west-1".to_string());
    let endpoint_url = env::var("S3_CACHE_ENDPOINT_URL").ok().filter(|s| !s.is_empty());
    let force_path_style = env::var("S3_CACHE_FORCE_PATH_STYLE")
        .map(|v| v.trim().to_lowercase() == "true")
        .unwrap_or(false);
    let kms_key_id = env::var("S3_CACHE_KMS_KEY_ID").ok().filter(|s| !s.is_empty());
    
    let encryption_key = env::var("S3_CACHE_ENCRYPTION_KEY")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(|b64| {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .expect("S3_CACHE_ENCRYPTION_KEY is not valid base64");
            let key: [u8; 32] = bytes
                .try_into()
                .expect("S3_CACHE_ENCRYPTION_KEY must decode to exactly 32 bytes");
            key
        });

    let s3_config = S3CacheConfig {
        enabled: true,
        bucket,
        region,
        endpoint_url,
        force_path_style,
        kms_key_id,
        encryption_key,
    };

    // 5. Build S3 client via s3_cache_layer::build_s3_client()
    let (_s3_client, bucket_name) = s3_cache_layer::build_s3_client(&s3_config).await?;
    tracing::info!(bucket = %bucket_name, "S3 Client successfully initialized");

    // 6. TODO: Set up tokio-cron-scheduler for 1x/day execution
    // 7. TODO: Batch-fetch logic (awaiting final decision)
    todo!("Batch fetch strategy pending decision")
}
