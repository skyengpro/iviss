use crate::dto::search_vehicle::VehicleSearchResult;
use crate::utils::plate_format::{self, PlateCategory};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{config::Region, primitives::ByteStream, types::ServerSideEncryption};
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
    /// Optional AWS KMS key ARN for server-side encryption (SSE-KMS).
    /// When set, S3 encrypts objects at rest using this customer-managed key.
    /// When `None`, S3 uses its default encryption (SSE-S3 / bucket default).
    pub kms_key_id: Option<String>,
    /// Optional 32-byte AES-256-GCM key for client-side encryption.
    /// When set, the JSON payload is encrypted in-memory before upload so that
    /// S3 (and AWS) never sees plaintext vehicle data.
    /// When `None`, the payload is uploaded as plain JSON (suitable for local dev).
    pub encryption_key: Option<[u8; 32]>,
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
    /// SSE-KMS key ARN (server-side encryption layer).
    kms_key_id: Option<String>,
    /// AES-256-GCM key (client-side encryption layer).
    encryption_key: Option<[u8; 32]>,
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
            kms_key_id: config.kms_key_id.clone(),
            encryption_key: config.encryption_key,
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
        // duplicate writes under high concurrency on the same plate.
        // Rolled back on failure so the next request can retry.
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

        let raw_bytes = output
            .body
            .collect()
            .await
            .context("failed to collect vehicle cache object body")?
            .into_bytes();

        // --- Client-side decryption (reverse of Option D layer) ---
        let json_bytes = match &self.encryption_key {
            Some(aes_key) => payload_crypto::decrypt(aes_key, raw_bytes.as_ref())
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

// ---------------------------------------------------------------------------
// Client-side AES-256-GCM encryption helpers
// ---------------------------------------------------------------------------

/// Encrypt / decrypt vehicle cache payloads using AES-256-GCM.
///
/// The wire format is simple and self-contained:
///
/// ```text
/// [ 12-byte random nonce | ciphertext + 16-byte GCM auth tag ]
/// ```
///
/// The nonce is generated fresh for every `encrypt` call via the OS CSPRNG.
mod payload_crypto {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Key, Nonce,
    };
    use anyhow::{anyhow, ensure, Result};
    use rand::RngCore;

    /// 96-bit nonce as recommended by NIST SP 800-38D for AES-GCM.
    const NONCE_LEN: usize = 12;

    /// Encrypt `plaintext` and return `nonce || ciphertext`.
    pub fn encrypt(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
        // `From<[u8; N]>` is stable and not deprecated — preferred over `from_slice`.
        let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key_bytes));

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow!("AES-256-GCM encryption failed: {e}"))?;

        let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Decrypt data previously produced by [`encrypt`].
    pub fn decrypt(key_bytes: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
        ensure!(
            data.len() > NONCE_LEN,
            "encrypted payload is too short ({} bytes)",
            data.len()
        );

        let (nonce_slice, ciphertext) = data.split_at(NONCE_LEN);
        // `split_at(NONCE_LEN)` guarantees the left part is exactly NONCE_LEN bytes.
        let nonce_arr: [u8; NONCE_LEN] = nonce_slice
            .try_into()
            .expect("split_at guarantees exactly NONCE_LEN bytes");
        let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key_bytes));
        let nonce = Nonce::from(nonce_arr);

        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow!("AES-256-GCM decryption failed: {e}"))
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
        // State plates (CA / AN prefix)
        assert_eq!(cache_partition_for_plate("CA1234A"), "others");
        // NationalSecurity
        assert_eq!(cache_partition_for_plate("SN1234"), "others");
        // Diplomatic
        assert_eq!(cache_partition_for_plate("CMD02RC521"), "others");
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = [0xABu8; 32];
        let plaintext = b"{\"plate\":\"LT893DK\",\"owner\":\"test\"}";

        let encrypted = payload_crypto::encrypt(&key, plaintext).unwrap();

        // Encrypted output must differ from plaintext.
        assert_ne!(encrypted, plaintext);
        // Must be longer than plaintext (nonce + GCM tag overhead).
        assert!(encrypted.len() > plaintext.len());

        let decrypted = payload_crypto::decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = [0xABu8; 32];
        let wrong_key = [0xCDu8; 32];
        let plaintext = b"secret vehicle data";

        let encrypted = payload_crypto::encrypt(&key, plaintext).unwrap();
        let result = payload_crypto::decrypt(&wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_rejects_short_payload() {
        let key = [0xABu8; 32];
        // 12 bytes or fewer is too short (need nonce + at least 1 byte ciphertext).
        let result = payload_crypto::decrypt(&key, &[0u8; 12]);
        assert!(result.is_err());
    }
}
