use crate::config::S3CacheConfig;
use crate::dto::search_vehicle::VehicleSearchResult;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{config::Region, primitives::ByteStream};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEDUP_TTL_SECS: u64 = 8 * 60 * 60;
const DEDUP_MAX_CAPACITY: u64 = 50_000;

pub struct CachedVehicleData {
    pub data: VehicleSearchResult,
    pub cached_at: String,
}

#[async_trait]
pub trait VehicleDataCache: Send + Sync {
    async fn store_vehicle_data(&self, plate: &str, data: &VehicleSearchResult) -> Result<bool>;
    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>>;
}

pub struct S3VehicleDataCache {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
    dedup_cache: Cache<String, ()>,
}

#[derive(Serialize, Deserialize)]
struct CachedEntry {
    data: VehicleSearchResult,
    cached_at: String,
}

impl S3VehicleDataCache {
    pub async fn from_config(config: &S3CacheConfig) -> Result<Self> {
        let bucket = config
            .bucket
            .clone()
            .context("S3_CACHE_BUCKET must be set when S3 cache is enabled")?;
        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(config.region.clone())))
                .or_default_provider()
                .or_else("eu-west-1");

        let mut aws_config =
            aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
        if let Some(endpoint_url) = &config.endpoint_url {
            aws_config = aws_config.endpoint_url(endpoint_url.clone());
        }

        let shared_config = aws_config.load().await;
        let mut s3_config = aws_sdk_s3::config::Builder::from(&shared_config);
        if config.force_path_style {
            s3_config = s3_config.force_path_style(true);
        }

        Ok(Self {
            client: aws_sdk_s3::Client::from_conf(s3_config.build()),
            bucket,
            prefix: normalize_prefix(&config.prefix),
            dedup_cache: Cache::builder()
                .max_capacity(DEDUP_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(DEDUP_TTL_SECS))
                .build(),
        })
    }

    fn object_key(&self, plate: &str) -> Result<String> {
        if !plate.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(anyhow!(
                "vehicle cache plate key contains invalid characters"
            ));
        }

        Ok(format!("{}{}.json", self.prefix, plate))
    }
}

#[async_trait]
impl VehicleDataCache for S3VehicleDataCache {
    async fn store_vehicle_data(&self, plate: &str, data: &VehicleSearchResult) -> Result<bool> {
        if self.dedup_cache.get(plate).await.is_some() {
            return Ok(false);
        }

        let key = self.object_key(plate)?;
        let entry = CachedEntry {
            data: data.clone(),
            cached_at: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .context("failed to format vehicle cache timestamp")?,
        };
        let body = serde_json::to_vec(&entry).context("failed to serialize vehicle cache entry")?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type("application/json")
            .body(ByteStream::from(body))
            .send()
            .await
            .context("failed to write vehicle cache object")?;

        self.dedup_cache.insert(plate.to_string(), ()).await;
        Ok(true)
    }

    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>> {
        let key = self.object_key(plate)?;
        let output = match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(output) => output,
            Err(error) if is_not_found_error(&error) => return Ok(None),
            Err(error) => return Err(error).context("failed to read vehicle cache object"),
        };

        let bytes = output
            .body
            .collect()
            .await
            .context("failed to collect vehicle cache object body")?
            .into_bytes();
        let entry: CachedEntry =
            serde_json::from_slice(bytes.as_ref()).context("failed to deserialize cache entry")?;

        Ok(Some(CachedVehicleData {
            data: entry.data,
            cached_at: entry.cached_at,
        }))
    }
}

fn normalize_prefix(prefix: &str) -> String {
    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix}/")
    }
}

fn is_not_found_error(error: &impl std::fmt::Display) -> bool {
    let message = error.to_string();
    message.contains("NoSuchKey")
        || message.contains("NotFound")
        || message.contains("404")
        || message.contains("status code: 404")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_prefix_keeps_single_trailing_slash() {
        assert_eq!(normalize_prefix("vehicle-cache"), "vehicle-cache/");
        assert_eq!(normalize_prefix("/vehicle-cache/"), "vehicle-cache/");
        assert_eq!(normalize_prefix(""), "");
    }
}
