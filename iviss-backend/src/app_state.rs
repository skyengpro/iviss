use crate::app_cache::AppCache;
use crate::config::Config;
use crate::db::DbPool;
use crate::services::email_provider::EmailProvider;
use crate::services::email_service::EmailService;
use crate::services::jwt_service::JwtService;
use crate::services::otp_service::OtpService;
use crate::services::sms_provider::SmsProvider;
use crate::services::vehicle_client_service::VehicleApiServise;
use std::sync::Arc;
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub app_cache: Arc<AppCache>,
    pub otp_svc: Arc<OtpService>,
    pub email_svc: Arc<EmailService>,
    pub jwt_svc: Arc<JwtService>,
    pub jwt_public_key_pem: String,
    pub vehicle_api_svc: Arc<VehicleApiServise>,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        app_cache: Arc<AppCache>,
        sms_pvd: Arc<dyn SmsProvider>,
        email_pvd: Arc<dyn EmailProvider>,
        config: &Config,
    ) -> Self {
        let jwt_svc = JwtService::new(&config.jwt_private_key_pem)
            .expect("Failed to parse JWT private key PEM at startup");

        let otp_svc = OtpService::new(
            app_cache.clone(),
            sms_pvd.clone(),
            config.activation_code_pepper.clone(),
        );
        let email_svc = EmailService::new(email_pvd.clone());
        Self {
            db: db_pool,
            app_cache,
            otp_svc: Arc::new(otp_svc),
            email_svc: Arc::new(email_svc),
            jwt_svc: Arc::new(jwt_svc),
            jwt_public_key_pem: config.jwt_public_key_pem.clone(),
            vehicle_api_svc: Arc::new(VehicleApiServise::new(
                config.vehicle_api_credentials.clone(),
            )),
        }
    }
}
