use crate::db::{redis::RedisPool, DbPool};

pub struct AppState {
    pub db: DbPool,
    pub redis: RedisPool,
}

impl AppState {
    pub fn new(db_pool: DbPool, redis_pool: RedisPool) -> Self {
        Self {
            db: db_pool,
            redis: redis_pool,
        }
    }
}
