use crate::app_cache::AppCache;
use crate::config::Config;
use crate::db::DbPool;
use crate::services::email_provider::EmailProvider;
use crate::services::email_service::EmailService;
use crate::services::jwt_service::JwtService;
use crate::services::otp_service::OtpService;
use crate::services::sms_provider::SmsProvider;
use crate::services::vehicle_client_service::VehicleApiService;
use crate::telemetry::TelemetryHandle;
use anyhow::Context;
use std::sync::atomic::AtomicI64;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub app_cache: Arc<AppCache>,
    pub otp_svc: Arc<OtpService>,
    pub email_svc: Arc<EmailService>,
    pub jwt_svc: Arc<JwtService>,
    pub jwt_public_key_pem: String,
    pub otp_via_email: bool,
    pub vehicle_api_svc: Arc<VehicleApiService>,
    pub telemetry: Arc<TelemetryHandle>,
    pub active_sessions: Arc<AtomicI64>,
}

impl AppState {
    pub async fn new(
        db_pool: DbPool,
        app_cache: Arc<AppCache>,
        sms_pvd: Arc<dyn SmsProvider>,
        email_pvd: Arc<dyn EmailProvider>,
        config: &Config,
        telemetry: Arc<TelemetryHandle>,
    ) -> anyhow::Result<Self> {
        let jwt_svc = JwtService::new(&config.jwt_private_key_pem)
            .context("failed to parse JWT private key PEM at startup")?;

        let otp_svc = OtpService::new(
            app_cache.clone(),
            sms_pvd.clone(),
            email_pvd.clone(),
            config.activation_code_pepper.clone(),
            config.otp_via_email,
        );
        let email_svc = EmailService::new(email_pvd.clone());
        let vehicle_api_svc = VehicleApiService::new(config.vehicle_api_credentials.clone())
            .context("failed to initialize vehicle API service")?;

        // Count current active sessions from non-revoked, non-expired refresh tokens.
        let active_count: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM refresh_tokens WHERE revoked = FALSE AND expires_at > NOW()",
        )
        .fetch_one(&db_pool)
        .await
        .unwrap_or(0);

        let active_sessions = Arc::new(AtomicI64::new(active_count));
        metrics::gauge!("iviss_active_sessions").set(active_count as f64);

        Ok(Self {
            db: db_pool,
            app_cache,
            otp_svc: Arc::new(otp_svc),
            email_svc: Arc::new(email_svc),
            jwt_svc: Arc::new(jwt_svc),
            jwt_public_key_pem: config.jwt_public_key_pem.clone(),
            otp_via_email: config.otp_via_email,
            vehicle_api_svc: Arc::new(vehicle_api_svc),
            telemetry,
            active_sessions,
        })
    }
}
