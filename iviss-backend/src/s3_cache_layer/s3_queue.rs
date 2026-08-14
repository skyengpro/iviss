use crate::s3_cache_layer::types::{
    org_plate_from_key, org_prefix, retry_queue_key, unregistered_key, QueueMarker,
    RETRY_QUEUE_PREFIX, UNREGISTERED_PREFIX,
};
use anyhow::{Context, Result};
use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

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

pub async fn enqueue_plate(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    org_id: Uuid,
    plate: &str,
) -> Result<()> {
    put_marker(client, bucket, retry_queue_key(org_id, plate)?, plate).await
}

pub async fn mark_unregistered(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    org_id: Uuid,
    plate: &str,
) -> Result<()> {
    put_marker(client, bucket, unregistered_key(org_id, plate)?, plate).await
}

/// A listed marker parsed from an org-partitioned key.
pub struct ListedMarker {
    pub organization_id: Uuid,
    pub plate_number: String,
    pub last_modified: Option<time::OffsetDateTime>,
}

/// Paginated `ListObjectsV2` over `prefix`, parsing every key with `org_plate_from_key`.
/// Keys that do not match the org-partitioned format are silently skipped.
async fn list_markers_org_partitioned(
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
            let Some((organization_id, plate_number)) = org_plate_from_key(key, prefix) else {
                continue;
            };
            let last_modified = object
                .last_modified()
                .and_then(|dt| time::OffsetDateTime::from_unix_timestamp(dt.secs()).ok());
            markers.push(ListedMarker {
                organization_id,
                plate_number,
                last_modified,
            });
            if markers.len() >= max {
                return Ok(markers);
            }
        }

        continuation_token = output.next_continuation_token().map(String::from);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(markers)
}

pub async fn list_unregistered_markers(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    org_id: Uuid,
    max: usize,
) -> Result<Vec<ListedMarker>> {
    list_markers_org_partitioned(
        client,
        bucket,
        &org_prefix(UNREGISTERED_PREFIX, org_id),
        max,
    )
    .await
}

pub async fn list_all_unregistered_markers(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    max: usize,
) -> Result<Vec<ListedMarker>> {
    list_markers_org_partitioned(client, bucket, UNREGISTERED_PREFIX, max).await
}

/// Lists the retry queue across all orgs; returns `(org_id, plate)` pairs.
pub async fn list_queued_markers(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    max: usize,
) -> Result<Vec<(Uuid, String)>> {
    Ok(
        list_markers_org_partitioned(client, bucket, RETRY_QUEUE_PREFIX, max)
            .await?
            .into_iter()
            .map(|m| (m.organization_id, m.plate_number))
            .collect(),
    )
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

/// Deletes a retry-queue marker. Only targets `retry-queue/` — no delete IAM grant on `vehicle-cache/`.
pub async fn remove_marker(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    org_id: Uuid,
    plate: &str,
) -> Result<()> {
    debug_assert!(
        !plate.contains('/'),
        "remove_marker: plate must not contain path separators"
    );
    delete_marker(client, bucket, retry_queue_key(org_id, plate)?).await
}
