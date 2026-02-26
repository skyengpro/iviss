use anyhow::{Context, Result};
use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

pub type RedisPool = Pool;

pub async fn initialize_redis_pool(redis_url: &str) -> Result<RedisPool> {
    let cfg = RedisConfig::from_url(redis_url);

    let pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .context("Failed to create Redis connection pool")?;

    // Verify connection
    let mut conn = pool
        .get()
        .await
        .context("Failed to connect to Redis — is Redis running?")?;

    redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .context("Redis PING failed")?;

    Ok(pool)
}
