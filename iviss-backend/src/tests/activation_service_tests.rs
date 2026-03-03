use crate::db::RedisPool;
use crate::services::activation_service::{ActivationEntry, ActivationService};
use crate::services::sms_provider::MockSmsProvider;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config as RedisConfig, Runtime};
    use redis::AsyncCommands;
    use std::sync::Arc;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::redis::Redis;

    // ─────────────────────────────────────────
    // Helper — spin up Redis container + pool
    // ─────────────────────────────────────────
    async fn setup_redis_pool(port: u16) -> RedisPool {
        let url = format!("redis://127.0.0.1:{}", port);
        let cfg = RedisConfig::from_url(url);
        cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
    }

    const TEST_PEPPER: &str = "test_pepper_for_activation_code_hashing_must_be_32_chars_long";

    fn make_service(pool: RedisPool) -> ActivationService {
        ActivationService::new(pool, Arc::new(MockSmsProvider), TEST_PEPPER.to_string())
    }

    // ─────────────────────────────────────────
    // Pure Tests — Without Redis
    // ─────────────────────────────────────────

    #[test]
    fn test_generate_code_is_6_digits() {
        let svc = ActivationService::new(
            // Mock pool — not used in this test
            deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
                .create_pool(Some(Runtime::Tokio1))
                .unwrap(),
            Arc::new(MockSmsProvider),
            TEST_PEPPER.to_string(),
        );

        for _ in 0..100 {
            let code = svc.generate_code();
            assert_eq!(code.len(), 6, "Code must be exactly 6 chars");
            assert!(
                code.chars().all(|c| c.is_ascii_digit()),
                "Code must be numeric"
            );
        }
    }

    #[test]
    fn test_hash_code_is_deterministic() {
        let svc = make_service(
            deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
                .create_pool(Some(Runtime::Tokio1))
                .unwrap(),
        );
        let hash1 = svc.hash_code("123456");
        let hash2 = svc.hash_code("123456");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_code_different_inputs() {
        let svc = make_service(
            deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
                .create_pool(Some(Runtime::Tokio1))
                .unwrap(),
        );
        let hash1 = svc.hash_code("123456");
        let hash2 = svc.hash_code("654321");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_code_zero_padded() {
        // "000001" et "1" doivent donner des hash différents
        let svc = make_service(
            deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
                .create_pool(Some(Runtime::Tokio1))
                .unwrap(),
        );
        let hash1 = svc.hash_code("000001");
        let hash2 = svc.hash_code("1");
        assert_ne!(hash1, hash2);
    }

    // ─────────────────────────────────────────
    // Tests with Redis (testcontainers)
    // ─────────────────────────────────────────

    #[tokio::test]
    async fn test_generate_and_store_sets_key_in_redis() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();
        let code = svc.generate_and_store(&user_id).await.unwrap();

        // Verify that the key exists in Redis
        let mut conn = pool.get().await.unwrap();
        let raw: Option<String> = conn.get(format!("activation:{}", user_id)).await.unwrap();

        assert!(raw.is_some(), "Key must exist in Redis after generation");

        // Verify that the stored hash matches the returned code
        let entry: ActivationEntry = serde_json::from_str(&raw.unwrap()).unwrap();
        assert_eq!(entry.code_hash, svc.hash_code(&code));
        assert_eq!(entry.attempts, 0);
    }

    #[tokio::test]
    async fn test_validate_correct_code_succeeds() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();
        let code = svc.generate_and_store(&user_id).await.unwrap();

        let result = svc.validate(&user_id, &code).await;
        assert!(result.is_ok(), "Valid code must succeed");
    }

    #[tokio::test]
    async fn test_validate_wrong_code_increments_attempts() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();
        svc.generate_and_store(&user_id).await.unwrap();

        // Wrong code
        let _ = svc.validate(&user_id, "000000").await;

        let mut conn = pool.get().await.unwrap();
        let raw: Option<String> = conn.get(format!("activation:{}", user_id)).await.unwrap();
        let entry: ActivationEntry = serde_json::from_str(&raw.unwrap()).unwrap();

        assert_eq!(entry.attempts, 1, "Attempts must be incremented");
    }

    /*  #[tokio::test]
    async fn test_validate_max_attempts_invalidates_code() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();
        svc.generate_and_store(&user_id).await.unwrap();

        // 5 wrong attempts
        for _ in 0..5 {
            let _ = svc.validate(&user_id, "000000").await;
        }

        // The key must be deleted
        let mut conn = pool.get().await.unwrap();
        let raw: Option<String> = conn
            .get(format!("activation:{}", user_id))
            .await
            .unwrap();

        assert!(raw.is_none(), "Key must be deleted after max attempts");
    } */

    #[tokio::test]
    async fn test_validate_deletes_key_after_success() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();
        let code = svc.generate_and_store(&user_id).await.unwrap();

        svc.validate(&user_id, &code).await.unwrap();

        // The key must be deleted after success (single use)
        let mut conn = pool.get().await.unwrap();
        let raw: Option<String> = conn.get(format!("activation:{}", user_id)).await.unwrap();

        assert!(
            raw.is_none(),
            "Key must be deleted after successful validation"
        );
    }

    #[tokio::test]
    async fn test_validate_expired_code_returns_error() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();

        // Simulates an expired code — key does not exist
        let result = svc.validate(&user_id, "123456").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expired or not found"));
    }

    #[tokio::test]
    async fn test_generate_overwrites_existing_code() {
        let container = Redis::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(6379).await.unwrap();
        let pool = setup_redis_pool(port).await;
        let svc = make_service(pool.clone());

        let user_id = Uuid::new_v4();

        let code1 = svc.generate_and_store(&user_id).await.unwrap();
        let code2 = svc.generate_and_store(&user_id).await.unwrap();

        // The new code must be valid, the old one not
        let result_old = svc.validate(&user_id, &code1).await;
        // Regenerates because validate deleted the key if code1 == code2 (rare)
        if code1 != code2 {
            assert!(
                result_old.is_err(),
                "Old code must be invalid after regeneration"
            );
        }

        let code3 = svc.generate_and_store(&user_id).await.unwrap();
        let result_new = svc.validate(&user_id, &code3).await;
        assert!(result_new.is_ok(), "New code must be valid");
    }
}
