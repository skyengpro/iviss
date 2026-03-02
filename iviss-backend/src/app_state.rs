use crate::db::{redis::RedisPool, DbPool};
use crate::services::sms_provider::SmsProvider;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub redis: RedisPool,
    pub sms_pvd: Arc<dyn SmsProvider>,
}

impl AppState {
    pub fn new(db_pool: DbPool, redis_pool: RedisPool, sms_pvd: Arc<dyn SmsProvider>) -> Self {
        Self {
            db: db_pool,
            redis: redis_pool,
            sms_pvd,
        }
    }
}
