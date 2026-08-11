//! S3 Cache Sync Service
//!
//! Drains `retry-queue/` markers left behind by write-through failures on
//! Backend A: on each ping it checks the queue and the external API's health,
//! then — only when both are non-empty and healthy — drains the queue against
//! `ExternalDataSource`, writing hits to `vehicle-cache/` and misses to
//! `unregistered/`.
//!
//! Build: cargo build --bin s3-cache-sync --no-default-features

use iviss_backend::external_services::vehicle_client::{
    ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials, VehicleApiService,
};
use iviss_backend::external_services::{ExternalDataSource, ExternalServiceError, HealthStatus};
use iviss_backend::s3_cache_layer::s3_writer::write_vehicle_data;
use iviss_backend::s3_cache_layer::types::RETRY_QUEUE_PREFIX;
use iviss_backend::s3_cache_layer::{self, S3CacheConfig};
use std::env;
use std::time::Duration;
use tokio::time::Instant;

const DRAIN_WINDOW: Duration = Duration::from_secs(60 * 60);
const IDLE_BETWEEN_WINDOWS: Duration = Duration::from_secs(2 * 60 * 60);
const PING_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_CONSECUTIVE_FAILURES: u32 = 5;

/// How many retry-queue markers to list per page while draining.
const DRAIN_PAGE_SIZE: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting S3 Cache Sync Service...");

    dotenvy::dotenv().ok();

    let api_credentials = load_vehicle_api_credentials();
    tracing::info!(base_url = %api_credentials.base_url, "Vehicle API Service base URL loaded");
    let vehicle_api_svc = VehicleApiService::new(api_credentials)?;

    let s3_config = load_s3_cache_config();
    let kms_key_id = s3_config.kms_key_id.clone();
    let encryption_key = s3_config.encryption_key;
    let (s3_client, bucket_name) = s3_cache_layer::build_s3_client(&s3_config).await?;
    tracing::info!(bucket = %bucket_name, "S3 Client successfully initialized");

    let drain_window = duration_from_env("SYNC_WINDOW_SECS", DRAIN_WINDOW);
    let idle_between_windows = duration_from_env("SYNC_IDLE_SECS", IDLE_BETWEEN_WINDOWS);
    let ping_interval = duration_from_env("SYNC_PING_INTERVAL_SECS", PING_INTERVAL);
    let max_consecutive_failures =
        u32_from_env("SYNC_MAX_CONSECUTIVE_FAILURES", MAX_CONSECUTIVE_FAILURES);

    tracing::info!(
        drain_window_secs = drain_window.as_secs(),
        idle_secs = idle_between_windows.as_secs(),
        ping_interval_secs = ping_interval.as_secs(),
        max_consecutive_failures,
        "Starting sync cycle..."
    );

    loop {
        tracing::info!("Entering drain window");
        let window_deadline = Instant::now() + drain_window;

        while Instant::now() < window_deadline {
            tokio::time::sleep(ping_interval).await;

            run_ping_cycle(
                &vehicle_api_svc,
                &s3_client,
                &bucket_name,
                &kms_key_id,
                &encryption_key,
                max_consecutive_failures,
            )
            .await;
        }

        tracing::info!(
            idle_seconds = idle_between_windows.as_secs(),
            "Drain window elapsed; idling"
        );
        tokio::time::sleep(idle_between_windows).await;
    }
}

/// One ping tick: skip the health probe entirely when the queue is empty.
async fn run_ping_cycle(
    source: &impl ExternalDataSource,
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    kms_key_id: &Option<String>,
    encryption_key: &Option<[u8; 32]>,
    max_consecutive_failures: u32,
) {
    let queue_peek =
        s3_cache_layer::list_queued_plates(s3_client, bucket, RETRY_QUEUE_PREFIX, 1).await;
    match queue_peek {
        Ok(plates) if plates.is_empty() => {
            tracing::debug!("Retry queue empty; no probe emitted");
            return;
        }
        Ok(_) => {}
        Err(error) => {
            tracing::warn!(error = %error, "failed to peek retry queue; skipping this ping");
            return;
        }
    }

    match source.health_probe().await {
        HealthStatus::Unhealthy(reason) => {
            tracing::warn!(reason = %reason, "external API unhealthy; deferring drain");
        }
        HealthStatus::Healthy => {
            drain_queue(
                source,
                s3_client,
                bucket,
                kms_key_id,
                encryption_key,
                max_consecutive_failures,
            )
            .await;
        }
    }
}

/// Drains the retry queue until empty or `max_consecutive_failures` external
/// fetch failures in a row — a guard against "server up but /query broken",
/// which the health probe alone cannot detect.
async fn drain_queue(
    source: &impl ExternalDataSource,
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    kms_key_id: &Option<String>,
    encryption_key: &Option<[u8; 32]>,
    max_consecutive_failures: u32,
) {
    let mut consecutive_failures = 0u32;

    loop {
        let plates = match s3_cache_layer::list_queued_plates(
            s3_client,
            bucket,
            RETRY_QUEUE_PREFIX,
            DRAIN_PAGE_SIZE,
        )
        .await
        {
            Ok(plates) => plates,
            Err(error) => {
                tracing::warn!(error = %error, "failed to list retry queue during drain");
                return;
            }
        };

        if plates.is_empty() {
            tracing::info!("Retry queue drained");
            return;
        }

        for plate in plates {
            match source.fetch(&plate).await {
                Ok(iviss_backend::external_services::PartnerPayload::Vehicle {
                    vehicle, ..
                }) => {
                    consecutive_failures = 0;
                    if let Err(error) = write_vehicle_data(
                        s3_client,
                        bucket,
                        kms_key_id,
                        encryption_key,
                        &plate,
                        &vehicle,
                    )
                    .await
                    {
                        tracing::error!(plate = %plate, error = %error, "failed to write drained vehicle to cache; marker kept for retry");
                        continue;
                    }
                    if let Err(error) =
                        s3_cache_layer::remove_marker(s3_client, bucket, RETRY_QUEUE_PREFIX, &plate)
                            .await
                    {
                        tracing::error!(plate = %plate, error = %error, "failed to remove retry marker after successful write");
                    }
                }
                Err(ExternalServiceError::NotFound) => {
                    consecutive_failures = 0;
                    if let Err(error) =
                        s3_cache_layer::mark_unregistered(s3_client, bucket, &plate).await
                    {
                        tracing::error!(plate = %plate, error = %error, "failed to mark plate unregistered; marker kept for retry");
                        continue;
                    }
                    if let Err(error) =
                        s3_cache_layer::remove_marker(s3_client, bucket, RETRY_QUEUE_PREFIX, &plate)
                            .await
                    {
                        tracing::error!(plate = %plate, error = %error, "failed to remove retry marker after marking unregistered");
                    }
                }
                Err(ExternalServiceError::Unavailable(reason))
                | Err(ExternalServiceError::Protocol(reason)) => {
                    consecutive_failures += 1;
                    tracing::warn!(plate = %plate, reason = %reason, consecutive_failures, "external fetch failed during drain; marker kept for retry");

                    if consecutive_failures >= max_consecutive_failures {
                        tracing::warn!(
                            consecutive_failures,
                            "aborting drain cycle after too many consecutive failures"
                        );
                        return;
                    }
                }
            }
        }
    }
}

fn duration_from_env(var: &str, default: Duration) -> Duration {
    env::var(var)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(default)
}

fn u32_from_env(var: &str, default: u32) -> u32 {
    env::var(var)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default)
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
    let endpoint_url = env::var("S3_CACHE_ENDPOINT_URL")
        .ok()
        .filter(|s| !s.is_empty());
    let force_path_style = env::var("S3_CACHE_FORCE_PATH_STYLE")
        .map(|v| v.trim().to_lowercase() == "true")
        .unwrap_or(false);
    let kms_key_id = env::var("S3_CACHE_KMS_KEY_ID")
        .ok()
        .filter(|s| !s.is_empty());

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
