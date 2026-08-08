use crate::s3_cache_layer::crypto;
use crate::s3_cache_layer::types::{object_key, CachedEntry, CachedVehicleData};
use anyhow::{Context, Result};

/// Read vehicle data from S3, performing decryption if an encryption key is provided.
pub async fn read_vehicle_data(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    encryption_key: &Option<[u8; 32]>,
    plate: &str,
) -> Result<Option<CachedVehicleData>> {
    let key = object_key(plate)?;
    let output = match client.get_object().bucket(bucket).key(key).send().await {
        Ok(output) => output,
        Err(err) if err.as_service_error().is_some_and(|e| e.is_no_such_key()) => {
            return Ok(None);
        }
        Err(error) => anyhow::bail!("failed to read vehicle cache object: {error}"),
    };

    let raw_bytes = output
        .body
        .collect()
        .await
        .context("failed to collect vehicle cache object body")?
        .into_bytes();

    let json_bytes = match encryption_key {
        Some(aes_key) => crypto::decrypt(aes_key, raw_bytes.as_ref())
            .context("failed to decrypt vehicle cache payload")?,
        None => raw_bytes.to_vec(),
    };

    let entry: CachedEntry =
        serde_json::from_slice(&json_bytes).context("failed to deserialize cache entry")?;

    let cached_at = time::OffsetDateTime::parse(
        &entry.cached_at,
        &time::format_description::well_known::Rfc3339,
    )
    .context("failed to parse cached_at timestamp")?;

    Ok(Some(CachedVehicleData {
        plate_number: entry.plate_number,
        vehicle: entry.vehicle,
        cached_at,
    }))
}
