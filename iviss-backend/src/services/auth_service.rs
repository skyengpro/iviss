use crate::db::DbPools;
use crate::errors::AppError;

pub struct AuthService;

impl AuthService {
    pub async fn login() -> Result<String, AppError> {
        Ok("token".to_string())
    }
}
