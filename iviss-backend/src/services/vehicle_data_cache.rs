use crate::dto::search_vehicle::VehicleSearchResult;
use crate::utils::plate_format::{self, PlateCategory};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{config::Region, primitives::ByteStream};
use moka::future::Cache;
use serde::{Deserialize, Serialize};

const S3_CACHE_PREFIX: &str = "vehicle-cache/";
const REGION_CODES: &[&str] = &[
    "AD", "CE", "EN", "ES", "LT", "NO", "NW", "OU", "SU", "SW", "SO",
];
const OTHER_CACHE_PARTITION: &str = "others";

/// S3-compatible vehicle data cache configuration.
#[derive(Clone, Debug, Default)]
pub struct S3CacheConfig {
    pub enabled: bool,
    pub bucket: Option<String>,
    pub region: String,
    pub endpoint_url: Option<String>,
    pub force_path_style: bool,
}

pub struct CachedVehicleData {
    pub data: VehicleSearchResult,
    pub cached_at: time::OffsetDateTime,
}

#[async_trait]
pub trait VehicleDataCache: Send + Sync {
    async fn store_vehicle_data(&self, plate: &str, data: &VehicleSearchResult) -> Result<bool>;
    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>>;
}

pub struct S3VehicleDataCache {
    client: aws_sdk_s3::Client,
    bucket: String,
    dedup_cache: Cache<String, ()>,
}

#[derive(Serialize, Deserialize)]
struct CachedEntry {
    data: VehicleSearchResult,
    cached_at: String,
}

impl S3VehicleDataCache {
    /// Build from configuration.
    ///
    /// `dedup_cache` is created centrally in [`crate::app_cache::AppCache`] so
    /// that every in-memory cache in the application is visible in one place.
    pub async fn from_config(
        config: &S3CacheConfig,
        dedup_cache: Cache<String, ()>,
    ) -> Result<Self> {
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
            dedup_cache,
        })
    }

    fn object_key(&self, plate: &str) -> Result<String> {
        if !plate.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(anyhow!(
                "vehicle cache plate key contains invalid characters"
            ));
        }

        let partition = cache_partition_for_plate(plate);
        Ok(format!("{}{partition}/{plate}.json", S3_CACHE_PREFIX))
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

        // Insert into dedup cache BEFORE the S3 write to prevent concurrent
        // duplicate writes under high concurrency on the same plate.
        // Rolled back on failure so the next request can retry.
        self.dedup_cache.insert(plate.to_string(), ()).await;

        if let Err(error) = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type("application/json")
            .body(ByteStream::from(body))
            .send()
            .await
        {
            self.dedup_cache.invalidate(plate).await;
            anyhow::bail!("failed to write vehicle cache object: {error}");
        }

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
            Err(err) if err.as_service_error().is_some_and(|e| e.is_no_such_key()) => {
                return Ok(None);
            }
            Err(error) => anyhow::bail!("failed to read vehicle cache object: {error}"),
        };

        let bytes = output
            .body
            .collect()
            .await
            .context("failed to collect vehicle cache object body")?
            .into_bytes();
        let entry: CachedEntry =
            serde_json::from_slice(bytes.as_ref()).context("failed to deserialize cache entry")?;

        let cached_at = time::OffsetDateTime::parse(
            &entry.cached_at,
            &time::format_description::well_known::Rfc3339,
        )
        .context("failed to parse cached_at timestamp")?;

        Ok(Some(CachedVehicleData {
            data: entry.data,
            cached_at,
        }))
    }
}

fn cache_partition_for_plate(plate: &str) -> &str {
    let Some(found) = plate_format::classify(plate) else {
        return OTHER_CACHE_PARTITION;
    };

    match found.category {
        PlateCategory::CivilCemac
        | PlateCategory::CivilLegacy
        | PlateCategory::Trailer
        | PlateCategory::BikeCemac
        | PlateCategory::TestVehicle => plate
            .get(..2)
            .filter(|region| REGION_CODES.contains(region))
            .unwrap_or(OTHER_CACHE_PARTITION),
        PlateCategory::State
        | PlateCategory::Diplomatic
        | PlateCategory::Temporary
        | PlateCategory::Transit
        | PlateCategory::Postal
        | PlateCategory::SpecialInvestment
        | PlateCategory::NationalSecurity
        | PlateCategory::Military
        | PlateCategory::PostalTelecom
        | PlateCategory::GovernmentLegacy => OTHER_CACHE_PARTITION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_partition_routes_regional_plates_by_region_code() {
        assert_eq!(cache_partition_for_plate("LT893DK"), "LT");
        assert_eq!(cache_partition_for_plate("CE128BC"), "CE");
        assert_eq!(cache_partition_for_plate("NW777AB"), "NW");
        assert_eq!(cache_partition_for_plate("LTSR9652A"), "LT");
    }

    #[test]
    fn cache_partition_routes_special_formats_to_others() {
        assert_eq!(cache_partition_for_plate("CA1234A"), "others");
        assert_eq!(cache_partition_for_plate("SN1234"), "others");
        assert_eq!(cache_partition_for_plate("CMD02RC521"), "others");
        assert_eq!(cache_partition_for_plate("EN1234X"), "others");
        assert_eq!(cache_partition_for_plate("1234567"), "others");
    }
}
