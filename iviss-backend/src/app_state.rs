use crate::config::Config;
use crate::db::{redis::RedisPool, DbPool};
use crate::services::jwt_service::JwtService;
use crate::services::otp_service::OtpService;
use crate::services::sms_provider::SmsProvider;
use std::sync::Arc;
use crate::app_cache::AppCache;
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub redis: RedisPool,
    pub (crate)app_cache: Arc<AppCache>,
    pub otp_svc: Arc<OtpService>,
    pub jwt_svc: Arc<JwtService>,
    pub jwt_public_key_pem: String,
    pub shift_start_hour: u32,
    pub shift_end_hour: u32,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        redis_pool: RedisPool,
        app_cache: Arc<AppCache>,
        sms_pvd: Arc<dyn SmsProvider>,
        config: &Config,
    ) -> Self {
        let jwt_svc = JwtService::new(&config.jwt_private_key_pem)
            .expect("Failed to parse JWT private key PEM at startup");

        let otp_svc = OtpService::new(
            app_cache.clone(),
            sms_pvd.clone(),
            config.activation_code_pepper.clone(),
        );
        Self {
            db: db_pool,
            redis: redis_pool,
            app_cache,
            otp_svc: Arc::new(otp_svc),
            jwt_svc: Arc::new(jwt_svc),
            jwt_public_key_pem: config.jwt_public_key_pem.clone(),
            shift_start_hour: config.shift_start_hour,
            shift_end_hour: config.shift_end_hour,
        }
    }
}
