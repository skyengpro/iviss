use crate::db::DbPool;

pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db_pool: DbPool, jwt_secret: String) -> Self {
        Self {
            db: db_pool,
            jwt_secret,
        }
    }
}
