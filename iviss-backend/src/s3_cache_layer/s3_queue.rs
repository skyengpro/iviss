use crate::s3_cache_layer::types::{
    plate_from_key, retry_queue_key, unregistered_key, QueueMarker, RETRY_QUEUE_PREFIX,
    UNREGISTERED_PREFIX,
};
use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;

async fn put_marker(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: String,
    plate: &str,
) -> Result<()> {
    let marker = QueueMarker {
        plate_number: plate.to_string(),
        queued_at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .context("failed to format queue marker timestamp")?,
    };
    let body = serde_json::to_vec(&marker).context("failed to serialize queue marker")?;

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type("application/json")
        .body(ByteStream::from(body))
        .send()
        .await
        .map_err(|error| anyhow::anyhow!("failed to write queue marker: {error}"))?;

    Ok(())
}

pub async fn enqueue_plate(client: &aws_sdk_s3::Client, bucket: &str, plate: &str) -> Result<()> {
    put_marker(client, bucket, retry_queue_key(plate)?, plate).await
}

pub async fn mark_unregistered(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    plate: &str,
) -> Result<()> {
    put_marker(client, bucket, unregistered_key(plate)?, plate).await
}

/// A listed marker: the plate parsed from its key, plus the object's S3
/// `last_modified` — markers are write-once, so this doubles as `queued_at`
/// without needing a `GetObject` per entry.
pub struct ListedMarker {
    pub plate_number: String,
    pub last_modified: Option<time::OffsetDateTime>,
}

/// Paginated `ListObjectsV2` over a flat prefix, returning up to `max` entries.
async fn list_markers(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    prefix: &str,
    max: usize,
) -> Result<Vec<ListedMarker>> {
    let mut markers = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(token) = &continuation_token {
            request = request.continuation_token(token);
        }

        let output = request
            .send()
            .await
            .map_err(|error| anyhow::anyhow!("failed to list objects under {prefix}: {error}"))?;

        for object in output.contents() {
            let Some(key) = object.key() else { continue };
            if let Some(plate_number) = plate_from_key(key, prefix) {
                let last_modified = object
                    .last_modified()
                    .and_then(|dt| time::OffsetDateTime::from_unix_timestamp(dt.secs()).ok());
                markers.push(ListedMarker {
                    plate_number,
                    last_modified,
                });
                if markers.len() >= max {
                    return Ok(markers);
                }
            }
        }

        continuation_token = output.next_continuation_token().map(String::from);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(markers)
}

/// Paginated `ListObjectsV2` over a flat prefix, returning up to `max` plate numbers.
pub async fn list_queued_plates(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    prefix: &str,
    max: usize,
) -> Result<Vec<String>> {
    Ok(list_markers(client, bucket, prefix, max)
        .await?
        .into_iter()
        .map(|marker| marker.plate_number)
        .collect())
}

pub async fn list_unregistered_markers(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    max: usize,
) -> Result<Vec<ListedMarker>> {
    list_markers(client, bucket, UNREGISTERED_PREFIX, max).await
}

async fn delete_marker(client: &aws_sdk_s3::Client, bucket: &str, key: String) -> Result<()> {
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|error| anyhow::anyhow!("failed to delete queue marker: {error}"))?;

    Ok(())
}

/// `prefix` must be [`RETRY_QUEUE_PREFIX`] — never `vehicle-cache/`, which has no delete IAM grant.
pub async fn remove_marker(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    prefix: &str,
    plate: &str,
) -> Result<()> {
    debug_assert_eq!(
        prefix, RETRY_QUEUE_PREFIX,
        "remove_marker must only target the retry queue"
    );
    delete_marker(client, bucket, format!("{prefix}{plate}.json")).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::s3_cache_layer::types::{object_key, UNREGISTERED_PREFIX};

    #[test]
    fn plate_from_key_round_trips_through_retry_queue_key() {
        let key = retry_queue_key("LT893DK").unwrap();
        assert_eq!(
            plate_from_key(&key, RETRY_QUEUE_PREFIX),
            Some("LT893DK".to_string())
        );
    }

    #[test]
    fn plate_from_key_round_trips_through_unregistered_key() {
        let key = unregistered_key("CE128BC").unwrap();
        assert_eq!(
            plate_from_key(&key, UNREGISTERED_PREFIX),
            Some("CE128BC".to_string())
        );
    }

    #[test]
    fn plate_from_key_rejects_partitioned_vehicle_cache_keys() {
        // vehicle-cache/ keys have a partition segment; the flat parser must not match them.
        let key = object_key("LT893DK").unwrap();
        assert_eq!(plate_from_key(&key, RETRY_QUEUE_PREFIX), None);
    }
}
