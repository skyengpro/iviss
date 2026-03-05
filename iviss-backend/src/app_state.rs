#![allow(clippy::all, dead_code, unused_variables)]

use crate::db::{redis::RedisPool, DbPool};
use crate::services::sms_provider::SmsProvider;
use crate::services::jwt_service::JwtService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub redis: RedisPool,
    pub sms_pvd: Arc<dyn SmsProvider>,
    pub pepper: String,
    pub jwt_secret: String,
    pub jwt_service: Arc<JwtService>,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        redis_pool: RedisPool,
        sms_pvd: Arc<dyn SmsProvider>,
        pepper: String,
        jwt_secret: String,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            db: db_pool,
            redis: redis_pool,
            sms_pvd,
            pepper,
            jwt_secret,
            jwt_service,
        }
    }
}
