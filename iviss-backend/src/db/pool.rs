use sqlx::postgres::{PgPoolOptions, PgPool};
use anyhow::Result;
use std::time::Duration;

#[derive(Clone)]
pub struct DbPools {
    pub internal: PgPool,
    pub external: PgPool,
}

pub async fn initialize_pools(internal_url: &str, external_url: &str) -> Result<DbPools> {
    let internal = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(internal_url)
        .await?;

    let external = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(external_url)
        .await?;
        
    Ok(DbPools { internal, external })
}
