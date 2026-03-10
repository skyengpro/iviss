#![allow(clippy::all, dead_code, unused_variables)]

use crate::config::Config;
use crate::db::{redis::RedisPool, DbPool};
use crate::services::sms_provider::SmsProvider;
use std::sync::Arc;
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub redis: RedisPool,
    pub sms_pvd: Arc<dyn SmsProvider>,
    pub pepper: String,
    pub jwt_private_key_pem: String,
    pub jwt_public_key_pem: String,
    pub shift_start_hour: u32,
    pub shift_end_hour: u32,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        redis_pool: RedisPool,
        sms_pvd: Arc<dyn SmsProvider>,
        config: &Config,
    ) -> Self {
        Self {
            db: db_pool,
            redis: redis_pool,
            sms_pvd,
            pepper: config.activation_code_pepper.clone(),
            jwt_private_key_pem: config.jwt_private_key_pem.clone(),
            jwt_public_key_pem: config.jwt_public_key_pem.clone(),
            shift_start_hour: config.shift_start_hour,
            shift_end_hour: config.shift_end_hour,
        }
    }
}
