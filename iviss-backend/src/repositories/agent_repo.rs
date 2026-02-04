use crate::db::DbPools;
use crate::models::agent::Agent;
use crate::errors::AppError;

pub struct AgentRepo;

impl AgentRepo {
    pub async fn find_by_username(pool: &DbPools, username: &str) -> Result<Option<Agent>, AppError> {
        Ok(None)
    }
}
