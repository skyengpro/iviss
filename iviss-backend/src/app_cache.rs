use crate::errors::AppError;
use crate::queries::auth_queries;
use crate::queries::organization_queries;
use moka::future::Cache;
use moka::Expiry;
use std::time::{Duration, Instant};
use uuid::Uuid;

const RATE_LIMIT_TTL_SECS: u64 = 600; // 10 min
const NONCE_TTL_SECS: u64 = 60; // 1 min
const JTI_BLACKLIST_TTL_SECS: u64 = 180; // 3 min

#[derive(Clone, Debug)]
pub struct OtpEntry {
    pub code_hash: String,
    pub attempts: u8,
    pub expires_at: Instant,
}

struct OtpExpiry;

impl Expiry<Uuid, OtpEntry> for OtpExpiry {
    fn expire_after_create(
        &self,
        _key: &Uuid,
        value: &OtpEntry,
        _created_at: Instant,
    ) -> Option<Duration> {
        value.expires_at.checked_duration_since(Instant::now())
    }

    fn expire_after_update(
        &self,
        _key: &Uuid,
        value: &OtpEntry,
        _updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        value.expires_at.checked_duration_since(Instant::now())
    }
}

#[derive(Clone)]
pub struct AppCache {
    /// Key: user_id (Uuid)
    /// Valeur : OtpEntry { code_hash, attempts }
    pub otp_store: Cache<Uuid, OtpEntry>,
    /// Key: phone_number (String)
    /// Number of request in the last RATE_LIMIT_TTL_SECS time window
    pub rate_limit: Cache<String, u32>,
    /// Key: device_id (Uuid)
    /// Valeur : nonce base64 (String)
    pub refresh_nonce: Cache<Uuid, String>,
    /// Key: jti (String)
    /// Value: ()
    pub jti_blacklist: Cache<String, ()>,
    /// Key: organization_id (Uuid)
    /// Value: (shift_start_minute, shift_end_minute)
    pub org_shift_hours: Cache<Uuid, (u32, u32)>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            otp_store: Cache::builder()
                .max_capacity(5_000)
                .expire_after(OtpExpiry)
                .build(),
            rate_limit: Cache::builder()
                .max_capacity(15_000)
                .time_to_live(Duration::from_secs(RATE_LIMIT_TTL_SECS))
                .build(),
            refresh_nonce: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(NONCE_TTL_SECS))
                .build(),
            jti_blacklist: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(JTI_BLACKLIST_TTL_SECS))
                .build(),
            org_shift_hours: Cache::builder().max_capacity(50).build(),
        }
    }

    pub async fn cache_necessary_data_from_database(
        &self,
        db_pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> Result<(), AppError> {
        auth_queries::load_blacklisted_jtis_to_cache(db_pool, self).await?;
        organization_queries::load_organizations_work_time_to_cache(db_pool, self).await?;

        Ok(())
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}
