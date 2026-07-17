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
    pub async fn from_config(config: &S3CacheConfig) -> Result<Self> {
        let (client, bucket) = s3_cache_layer::build_s3_client(config).await?;
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
