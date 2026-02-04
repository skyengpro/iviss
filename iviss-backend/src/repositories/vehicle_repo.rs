use crate::db::DbPools;
use crate::models::vehicle::Vehicle;
use crate::errors::AppError;

pub struct VehicleRepo;

impl VehicleRepo {
    pub async fn find_by_plate(pool: &DbPools, plate: &str) -> Result<Option<Vehicle>, AppError> {
        Ok(None)
    }
}
