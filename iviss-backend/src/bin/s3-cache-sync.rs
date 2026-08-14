//! S3 Cache Sync Service
//!
//! Drains `retry-queue/` markers left behind by write-through failures on
//! Backend A: on each ping it checks the queue and the external API's health,
//! then — only when both are non-empty and healthy — drains the queue against
//! `ExternalDataSource`, writing hits to `vehicle-cache/` and misses to
//! `unregistered/`.
//!
//! Build: cargo build --bin s3-cache-sync --no-default-features

use anyhow::Context;
use iviss_backend::external_services::vehicle_client::{
    ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials, VehicleApiService,
};
use iviss_backend::external_services::{ExternalDataSource, ExternalServiceError, HealthStatus};
use iviss_backend::s3_cache_layer::s3_writer::write_vehicle_data;
use iviss_backend::s3_cache_layer::{self, S3CacheConfig};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

/// How long each drain window stays open per cycle.
const DRAIN_WINDOW: Duration = Duration::from_secs(60 * 60);
/// How long to sleep between drain windows.
const IDLE_BETWEEN_WINDOWS: Duration = Duration::from_secs(2 * 60 * 60);
/// How often to check the queue/health during a drain window.
const PING_INTERVAL: Duration = Duration::from_secs(5 * 60);
/// Consecutive failures allowed before aborting the current drain cycle.
const MAX_CONSECUTIVE_FAILURES: u32 = 5;

/// How many retry-queue markers to list per page while draining.
const DRAIN_PAGE_SIZE: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting S3 Cache Sync Service...");

    dotenvy::dotenv().ok();

    let vehicle_api_enabled = bool_from_env("ENABLE_VEHICLE_API", true);
    if !vehicle_api_enabled {
        tracing::warn!("Vehicle API disabled via ENABLE_VEHICLE_API; sync cycle will idle");
    }

    let api_credentials = load_vehicle_api_credentials();
    tracing::info!(base_url = %api_credentials.base_url, "Vehicle API Service base URL loaded");
    let vehicle_api_svc = VehicleApiService::new(api_credentials)?;

    let s3_config = load_s3_cache_config()?;
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
            tracing::info!("Ping interval elapsed; checking queue and external API health");

            run_ping_cycle(
                vehicle_api_enabled,
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

/// One ping tick: skip the health probe entirely when the queue is empty,
/// or entirely when the vehicle API is disabled via `ENABLE_VEHICLE_API`.
async fn run_ping_cycle(
    vehicle_api_enabled: bool,
    source: &impl ExternalDataSource,
    s3_client: &aws_sdk_s3::Client,
    bucket: &str,
    kms_key_id: &Option<String>,
    encryption_key: &Option<[u8; 32]>,
    max_consecutive_failures: u32,
) {
    if !vehicle_api_enabled {
        tracing::debug!("Vehicle API disabled; skipping ping cycle");
        return;
    }

    match s3_cache_layer::list_queued_markers(s3_client, bucket, 1).await {
        Ok(markers) if markers.is_empty() => {
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

/// Drains the retry queue until empty or `max_consecutive_failures` failures
/// in a row.
///
/// Markers are grouped by plate before fetching: if multiple orgs queued the
/// same plate, the external API is called once and the result is fanned out
/// to each org's prefix.
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
        let markers =
            match s3_cache_layer::list_queued_markers(s3_client, bucket, DRAIN_PAGE_SIZE).await {
                Ok(m) => m,
                Err(error) => {
                    tracing::warn!(error = %error, "failed to list retry queue during drain");
                    return;
                }
            };

        if markers.is_empty() {
            tracing::info!("Retry queue drained");
            return;
        }

        // Group by plate so each unique plate generates one external API call,
        // regardless of how many orgs queued it.
        let mut by_plate: HashMap<String, Vec<Uuid>> = HashMap::new();
        for (org_id, plate) in markers {
            by_plate.entry(plate).or_default().push(org_id);
        }

        'plate_loop: for (plate, orgs) in by_plate {
            match source.fetch(&plate).await {
                Ok(iviss_backend::external_services::PartnerPayload::Vehicle {
                    vehicle, ..
                }) => {
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
                        consecutive_failures += 1;
                        tracing::error!(plate, error = %error, consecutive_failures, "failed to write drained vehicle to cache; markers kept for retry");
                        if consecutive_failures >= max_consecutive_failures {
                            tracing::warn!(
                                consecutive_failures,
                                "aborting drain cycle after too many consecutive failures"
                            );
                            return;
                        }
                        continue 'plate_loop;
                    }

                    consecutive_failures = 0;
                    tracing::info!(plate, "successfully drained vehicle to cache");

                    for org_id in orgs {
                        if let Err(error) =
                            s3_cache_layer::remove_marker(s3_client, bucket, org_id, &plate).await
                        {
                            tracing::error!(plate, org_id = %org_id, error = %error, "failed to remove retry marker after successful write");
                        }
                    }
                }

                Err(ExternalServiceError::NotFound) => {
                    for org_id in orgs {
                        if let Err(error) =
                            s3_cache_layer::mark_unregistered(s3_client, bucket, org_id, &plate)
                                .await
                        {
                            consecutive_failures += 1;
                            tracing::error!(plate, org_id = %org_id, error = %error, consecutive_failures, "failed to mark plate unregistered; marker kept for retry");
                            if consecutive_failures >= max_consecutive_failures {
                                tracing::warn!(
                                    consecutive_failures,
                                    "aborting drain cycle after too many consecutive failures"
                                );
                                return;
                            }
                            continue 'plate_loop;
                        }

                        consecutive_failures = 0;
                        tracing::info!(plate, org_id = %org_id, "successfully marked plate unregistered");

                        if let Err(error) =
                            s3_cache_layer::remove_marker(s3_client, bucket, org_id, &plate).await
                        {
                            tracing::error!(plate, org_id = %org_id, error = %error, "failed to remove retry marker after marking unregistered");
                        }
                    }
                }

                Err(ExternalServiceError::Unavailable(reason))
                | Err(ExternalServiceError::Protocol(reason)) => {
                    consecutive_failures += 1;
                    tracing::warn!(plate, reason = %reason, consecutive_failures, "external fetch failed during drain; markers kept for retry");

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

fn bool_from_env(var: &str, default: bool) -> bool {
    env::var(var)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

fn load_vehicle_api_credentials() -> VehicleApiCredentials {
    let base_url = env::var("EXTERNAL_API_BASE_URL")
        .unwrap_or_else(|_| "https://test-api.iviss.gov".to_string());
    let username = env::var("EXTERNAL_API_USERNAME").unwrap_or_default();
    let password = env::var("EXTERNAL_API_PASSWORD").unwrap_or_default();

    let header_user = env::var("EXTERNAL_API_USER").unwrap_or_default();
    let header_lock_ndia = env::var("EXTERNAL_API_LOCK_NDIA").unwrap_or_default();
    let header_kindia = env::var("EXTERNAL_API_KINDIA").unwrap_or_default();
    let header_client = env::var("EXTERNAL_API_CLIENT").unwrap_or_default();
    let header_ctr = env::var("EXTERNAL_API_CTR").unwrap_or_default();
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

fn load_s3_cache_config() -> anyhow::Result<S3CacheConfig> {
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
        .map(|b64| -> anyhow::Result<[u8; 32]> {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .context("S3_CACHE_ENCRYPTION_KEY is not valid base64")?;
            let key: [u8; 32] = bytes.try_into().map_err(|_| {
                anyhow::anyhow!("S3_CACHE_ENCRYPTION_KEY must decode to exactly 32 bytes")
            })?;
            Ok(key)
        })
        .transpose()?;

    Ok(S3CacheConfig {
        enabled: true,
        bucket,
        region,
        endpoint_url,
        force_path_style,
        kms_key_id,
        encryption_key,
    })
}
