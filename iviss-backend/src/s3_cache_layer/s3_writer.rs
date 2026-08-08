use crate::dto::search_vehicle::VehicleInfo;
use crate::s3_cache_layer::crypto;
use crate::s3_cache_layer::types::{object_key, CachedEntry};
use anyhow::{Context, Result};
use aws_sdk_s3::{primitives::ByteStream, types::ServerSideEncryption};

/// Write vehicle data to S3, performing client-side encryption and server-side encryption if configured.
pub async fn write_vehicle_data(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    kms_key_id: &Option<String>,
    encryption_key: &Option<[u8; 32]>,
    plate: &str,
    vehicle_info: &VehicleInfo,
) -> Result<()> {
    let key = object_key(plate)?;

    let entry = CachedEntry {
        plate_number: plate.to_string(),
        vehicle: vehicle_info.clone(),
        cached_at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .context("failed to format vehicle cache timestamp")?,
    };

    let json_bytes =
        serde_json::to_vec(&entry).context("failed to serialize vehicle cache entry")?;

    let (body, content_type) = match encryption_key {
        Some(aes_key) => {
            let ciphertext = crypto::encrypt(aes_key, &json_bytes)
                .context("failed to encrypt vehicle cache payload")?;
            (ciphertext, "application/octet-stream")
        }
        None => (json_bytes, "application/json"),
    };

    let mut request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .body(ByteStream::from(body));

    if let Some(kms_key) = kms_key_id {
        request = request
            .server_side_encryption(ServerSideEncryption::AwsKms)
            .ssekms_key_id(kms_key);
    }

    request
        .send()
        .await
        .map_err(|error| anyhow::anyhow!("failed to write vehicle cache object: {error}"))?;

    Ok(())
}
