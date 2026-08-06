use crate::s3_cache_layer::{self, CachedVehicleData, S3CacheConfig};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait VehicleDataCache: Send + Sync {
    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>>;
}

pub struct S3VehicleDataCache {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
    pub encryption_key: Option<[u8; 32]>,
    pub kms_key_id: Option<String>,
}

impl S3VehicleDataCache {
    /// Build from configuration.
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
            client,
            bucket,
            encryption_key: config.encryption_key,
            kms_key_id: config.kms_key_id.clone(),
        })
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
        let json_bytes =
            serde_json::to_vec(&entry).context("failed to serialize vehicle cache entry")?;

        // --- Client-side encryption (Option D layer) ---
        let (body, content_type) = match &self.encryption_key {
            Some(aes_key) => {
                let ciphertext = payload_crypto::encrypt(aes_key, &json_bytes)
                    .context("failed to encrypt vehicle cache payload")?;
                (ciphertext, "application/octet-stream")
            }
            None => (json_bytes, "application/json"),
        };

        // Insert into dedup cache BEFORE the S3 write to prevent concurrent
        self.dedup_cache.insert(plate.to_string(), ()).await;

        // --- Server-side encryption (Option C layer) ---
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(body));

        if let Some(kms_key) = &self.kms_key_id {
            request = request
                .server_side_encryption(ServerSideEncryption::AwsKms)
                .ssekms_key_id(kms_key);
        }

        if let Err(error) = request.send().await {
            self.dedup_cache.invalidate(plate).await;
            tracing::error!(
                error = ?error,
                "failed to write vehicle cache object to S3"
            );
            anyhow::bail!("failed to write vehicle cache object: {error}");
        }

        Ok(true)
    }

    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>> {
        s3_cache_layer::s3_reader::read_vehicle_data(
            &self.client,
            &self.bucket,
            &self.encryption_key,
            plate,
        )
        .await
    }
}
