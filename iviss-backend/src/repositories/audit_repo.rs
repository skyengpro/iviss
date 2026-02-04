use crate::db::DbPools;
use crate::models::audit::AuditLog;
use crate::errors::AppError;

pub struct AuditRepo;

impl AuditRepo {
    pub async fn create(pool: &DbPools, log: &AuditLog) -> Result<(), AppError> {
        Ok(())
    }
}
