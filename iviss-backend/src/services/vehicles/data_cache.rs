use crate::dto::search_vehicle::VehicleInfo;
use crate::s3_cache_layer::{self, CachedVehicleData, S3CacheConfig};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UnregisteredPlate {
    pub organization_id: Uuid,
    pub plate_number: String,
    /// S3 object `last_modified` of the marker — markers are write-once, so
    /// this is effectively when the plate was marked unregistered.
    pub marked_at: Option<time::OffsetDateTime>,
}

pub enum UnregisteredScope {
    Organization(Uuid),
    AllTenants,
}

#[async_trait]
pub trait VehicleDataCache: Send + Sync {
    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>>;
    async fn store_vehicle_data(&self, plate: &str, vehicle: &VehicleInfo) -> Result<()>;
    async fn enqueue_retry(&self, org_id: Uuid, plate: &str) -> Result<()>;
    async fn list_unregistered(&self, scope: UnregisteredScope) -> Result<Vec<UnregisteredPlate>>;
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

    async fn store_vehicle_data(&self, plate: &str, vehicle: &VehicleInfo) -> Result<()> {
        s3_cache_layer::s3_writer::write_vehicle_data(
            &self.client,
            &self.bucket,
            &self.kms_key_id,
            &self.encryption_key,
            plate,
            vehicle,
        )
        .await
    }

    async fn enqueue_retry(&self, org_id: Uuid, plate: &str) -> Result<()> {
        s3_cache_layer::enqueue_plate(&self.client, &self.bucket, org_id, plate).await
    }

    async fn list_unregistered(&self, scope: UnregisteredScope) -> Result<Vec<UnregisteredPlate>> {
        let markers = match scope {
            UnregisteredScope::Organization(org_id) => {
                s3_cache_layer::list_unregistered_markers(
                    &self.client,
                    &self.bucket,
                    org_id,
                    usize::MAX,
                )
                .await?
            }
            UnregisteredScope::AllTenants => {
                s3_cache_layer::list_all_unregistered_markers(
                    &self.client,
                    &self.bucket,
                    usize::MAX,
                )
                .await?
            }
        };

        Ok(markers
            .into_iter()
            .map(|m| UnregisteredPlate {
                organization_id: m.organization_id,
                plate_number: m.plate_number,
                marked_at: m.last_modified,
            })
            .collect())
    }
}
