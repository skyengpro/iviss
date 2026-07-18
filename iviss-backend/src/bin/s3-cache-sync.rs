//! S3 Cache Sync Service
//!
//! Periodically fetches vehicle data from the external API and populates
//! the S3 cache layer. Designed to run as a long-lived service that runs
//! every 5 minutes (configured for development/schedule test time).
//!
//! Build: cargo build --bin s3-cache-sync --no-default-features

use iviss_backend::dto::search_vehicle::{OwnerInfo, VehicleInfo};
use iviss_backend::s3_cache_layer::{self, S3CacheConfig};
use iviss_backend::s3_cache_layer::types::PLATE_PREFIX_CODES;
use iviss_backend::s3_cache_layer::s3_writer::write_vehicle_data;
use iviss_backend::vehicle_client::{
    ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials, VehicleApiService,
};
use iviss_backend::vehicle_client::parser::split_brand_and_model;
use std::env;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Init tracing (minimal, stdout only — no OpenTelemetry)
    tracing_subscriber::fmt::init();
    tracing::info!("Starting S3 Cache Sync Service...");

    dotenvy::dotenv().ok();

    // 2. Load VehicleApiCredentials from env vars
    let api_credentials = load_vehicle_api_credentials();
    let api_base_url = api_credentials.base_url.clone();
    tracing::info!(base_url = %api_base_url, "Vehicle API Service base URL loaded");

    // 3. Build VehicleApiService (shared module)
    let vehicle_api_svc = VehicleApiService::new(api_credentials)?;

    // 4. Load S3CacheConfig from env vars
    let s3_config = load_s3_cache_config();
    let kms_key_id = s3_config.kms_key_id.clone();
    let encryption_key = s3_config.encryption_key;

    // 5. Build S3 client via s3_cache_layer::build_s3_client()
    let (s3_client, bucket_name) = s3_cache_layer::build_s3_client(&s3_config).await?;
    tracing::info!(bucket = %bucket_name, "S3 Client successfully initialized");

    // 6. Set up tokio interval for periodic execution (every 5 minutes)
    let interval_secs = env::var("SYNC_INTERVAL_SECS")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .unwrap_or(300);

    tracing::info!(interval_seconds = interval_secs, "Starting periodic sync loop...");
    let mut sync_interval = interval(Duration::from_secs(interval_secs));

    loop {
        sync_interval.tick().await;
        tracing::info!("Beginning S3 cache sync cycle...");

        let mut total_fetched = 0;
        let mut total_saved = 0;
        let mut total_errors = 0;

        for prefix in PLATE_PREFIX_CODES {
            tracing::debug!(prefix = %prefix, "Requesting batch for prefix");
            match vehicle_api_svc.fetch_batch(prefix).await {
                Ok(vehicles) => {
                    let count = vehicles.len();
                    total_fetched += count;
                    tracing::info!(prefix = %prefix, count = count, "Successfully fetched batch");

                    for ext_vehicle in vehicles {
                        // Normalize the plate number (uppercase, spaces removed)
                        let normalized_plate = ext_vehicle.plate_number.replace(' ', "").to_uppercase();
                        if normalized_plate.is_empty() {
                            tracing::warn!("Skipping vehicle with empty plate number");
                            continue;
                        }

                        // Map ExternalVehicle to VehicleInfo
                        let (brand, model) = split_brand_and_model(ext_vehicle.mark_and_type.as_deref());
                        let vehicle_info = VehicleInfo {
                            brand,
                            model,
                            year: None,
                            color: None,
                            engine_power: ext_vehicle.engine_power.clone(),
                            fuel_type: None,
                            chassis_number: ext_vehicle.chassis_number.clone(),
                            customs_status: ext_vehicle.customs_status.clone(),
                            owner: OwnerInfo {
                                name: ext_vehicle.owner_name.clone(),
                                address: None,
                                national_id: None,
                            },
                        };

                        // Write to S3 cache
                        match write_vehicle_data(
                            &s3_client,
                            &bucket_name,
                            &kms_key_id,
                            &encryption_key,
                            &normalized_plate,
                            &vehicle_info,
                        )
                        .await
                        {
                            Ok(_) => {
                                total_saved += 1;
                                tracing::debug!(plate = %normalized_plate, "Cached vehicle successfully in S3");
                            }
                            Err(e) => {
                                total_errors += 1;
                                tracing::error!(
                                    plate = %normalized_plate,
                                    error = %e,
                                    "Failed to save vehicle data to S3 cache"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    total_errors += 1;
                    tracing::error!(prefix = %prefix, error = %e, "Failed to fetch batch for prefix");
                }
            }
        }

        tracing::info!(
            fetched = total_fetched,
            saved = total_saved,
            errors = total_errors,
            "S3 cache sync cycle complete."
        );
    }
}

fn load_vehicle_api_credentials() -> VehicleApiCredentials {
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

    VehicleApiCredentials {
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
    }
}

fn load_s3_cache_config() -> S3CacheConfig {
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

    S3CacheConfig {
        enabled: true,
        bucket,
        region,
        endpoint_url,
        force_path_style,
        kms_key_id,
        encryption_key,
    }
}