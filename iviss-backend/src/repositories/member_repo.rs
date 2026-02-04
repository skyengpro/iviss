use crate::db::DbPools;
use crate::models::member::Member;
use crate::errors::AppError;

pub struct MemberRepo;

impl MemberRepo {
    pub async fn find_all(pool: &DbPools) -> Result<Vec<Member>, AppError> {
        Ok(vec![])
    }
}
