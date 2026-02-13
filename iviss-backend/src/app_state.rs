use crate::db::DbPool;

pub struct AppState {
    pub db: DbPool,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db: db_pool }
    }
}