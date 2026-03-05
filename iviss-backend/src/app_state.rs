#![allow(clippy::all, dead_code, unused_variables)]

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
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        redis_pool: RedisPool,
        sms_pvd: Arc<dyn SmsProvider>,
        pepper: String,
        jwt_private_key_pem: String,
        jwt_public_key_pem: String,
    ) -> Self {
        Self {
            db: db_pool,
            redis: redis_pool,
            sms_pvd,
            pepper,
            jwt_private_key_pem,
            jwt_public_key_pem,
        }
    }
}
