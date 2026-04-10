use moka::future::Cache;
use std::time::Duration;
use uuid::Uuid;


const OTP_TTL_SECS: u64        = 300;  // 5 min
const RATE_LIMIT_TTL_SECS: u64 = 600;  // 10 min
const NONCE_TTL_SECS: u64      = 60;   // 1 min
const JTI_BLACKLIST_TTL_SECS: u64 = 180; // 3 min

#[derive(Clone, Debug)]
pub (crate) struct OtpEntry {
    pub otp_hash: String,
    pub attempts: u8,
}
#[derive(Clone)]
pub (crate) struct AppCache {
    /// Key: user_id (Uuid)
    /// Valeur : OtpEntry { code_hash, attempts }
   pub  otp_store: Cache<Uuid, OtpEntry>,
    /// Key: phone_number (String)
    /// Number of request in the last RATE_LIMIT_TTL_SECS time window
    pub rate_limit: Cache<String, u32>,
    /// Key: device_id (Uuid)
    /// Valeur : nonce base64 (String)
    pub refresh_nonce: Cache<Uuid, String>,
    /// Key: jti (String)
    /// Value: ()
    pub jti_blacklist: Cache<String, ()>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            otp_store: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(OTP_TTL_SECS))
                .build(),
            rate_limit: Cache::builder()
                .max_capacity(30_000)
                .time_to_live(Duration::from_secs(RATE_LIMIT_TTL_SECS))
                .build(),
            refresh_nonce: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(NONCE_TTL_SECS))
                .build(),
            jti_blacklist: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(JTI_BLACKLIST_TTL_SECS))
                .build(),
        }
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}