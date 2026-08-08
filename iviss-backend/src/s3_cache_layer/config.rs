use anyhow::{Context, Result};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::config::Region;

/// S3-compatible vehicle data cache configuration.
#[derive(Clone, Debug, Default)]
pub struct S3CacheConfig {
    pub enabled: bool,
    pub bucket: Option<String>,
    pub region: String,
    pub endpoint_url: Option<String>,
    pub force_path_style: bool,
    /// Optional AWS KMS key ARN for server-side encryption (SSE-KMS).
    pub kms_key_id: Option<String>,
    /// Optional 32-byte AES-256-GCM key for client-side encryption.
    pub encryption_key: Option<[u8; 32]>,
}

/// Build an AWS SDK S3 Client and return it along with the configured bucket name.
pub async fn build_s3_client(config: &S3CacheConfig) -> Result<(aws_sdk_s3::Client, String)> {
    let bucket = config
        .bucket
        .clone()
        .context("S3_CACHE_BUCKET must be set when S3 cache is enabled")?;

    let region_provider = RegionProviderChain::first_try(Some(Region::new(config.region.clone())))
        .or_default_provider()
        .or_else("eu-west-1");

    let mut aws_config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if let Some(endpoint_url) = &config.endpoint_url {
        aws_config = aws_config.endpoint_url(endpoint_url.clone());
    }

    let shared_config = aws_config.load().await;
    let mut s3_config = aws_sdk_s3::config::Builder::from(&shared_config);
    if config.force_path_style {
        s3_config = s3_config.force_path_style(true);
    }

    Ok((aws_sdk_s3::Client::from_conf(s3_config.build()), bucket))
}
