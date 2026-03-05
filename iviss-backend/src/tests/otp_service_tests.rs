use crate::services::otp_service::OtpService;
use crate::services::sms_provider::MockSmsProvider;
use deadpool_redis::{Config as RedisConfig, Runtime};
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::Redis;
use uuid::Uuid;

// ─────────────────────────────────────────
// Helper
// ─────────────────────────────────────────

async fn setup_otp_service(port: u16) -> OtpService {
    let url = format!("redis://127.0.0.1:{}", port);
    let pool = RedisConfig::from_url(url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    OtpService::new(
        pool,
        Arc::new(MockSmsProvider),
        "test-pepper-that-is-at-least-32-chars!!".to_string(),
    )
}

// ─────────────────────────────────────────
// OTP generation & validation
// ─────────────────────────────────────────

#[tokio::test]
async fn test_request_otp_succeeds() {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let svc = setup_otp_service(port).await;

    let user_id = Uuid::new_v4();
    let result = svc.request_otp(&user_id, "+237600000000").await;
    assert!(result.is_ok(), "OTP request must succeed");
}

#[tokio::test]
async fn test_validate_otp_wrong_code_fails() {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let svc = setup_otp_service(port).await;

    let user_id = Uuid::new_v4();
    svc.request_otp(&user_id, "+237600000000").await.unwrap();

    let result = svc.validate_otp(&user_id, "000000").await;
    assert!(result.is_err(), "Wrong OTP must fail");
}

#[tokio::test]
async fn test_validate_otp_no_key_fails() {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let svc = setup_otp_service(port).await;

    // No OTP requested — key doesn't exist
    let result = svc.validate_otp(&Uuid::new_v4(), "123456").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expired or not found"));
}

#[tokio::test]
async fn test_otp_key_prefix_is_otp_not_activation() {
    use deadpool_redis::redis::AsyncCommands;

    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let url = format!("redis://127.0.0.1:{}", port);
    let pool = deadpool_redis::Config::from_url(url)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    let svc = OtpService::new(
        pool.clone(),
        Arc::new(MockSmsProvider),
        "test-pepper-that-is-at-least-32-chars!!".to_string(),
    );

    let user_id = Uuid::new_v4();
    svc.request_otp(&user_id, "+237600000000").await.unwrap();

    // Verify key uses "otp" prefix, not "activation"
    let mut conn = pool.get().await.unwrap();
    let otp_key: Option<String> = conn.get(format!("otp:{}", user_id)).await.unwrap();
    let activation_key: Option<String> = conn.get(format!("activation:{}", user_id)).await.unwrap();

    assert!(otp_key.is_some(), "otp:{user_id} key must exist");
    assert!(
        activation_key.is_none(),
        "activation:{user_id} must NOT exist"
    );
}

// ─────────────────────────────────────────
// Rate limiting
// ─────────────────────────────────────────

#[tokio::test]
async fn test_rate_limit_blocks_after_3_requests() {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let svc = setup_otp_service(port).await;

    let user_id = Uuid::new_v4();
    let phone = "+237600000001";

    // 3 allowed requests
    for _ in 0..3 {
        svc.request_otp(&user_id, phone).await.unwrap();
    }

    // 4th must be blocked
    let result = svc.request_otp(&user_id, phone).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Too many OTP requests"));
}

#[tokio::test]
async fn test_rate_limit_is_per_phone_number() {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let svc = setup_otp_service(port).await;

    let user_id = Uuid::new_v4();

    // Exhaust rate limit for phone 1
    for _ in 0..3 {
        svc.request_otp(&user_id, "+237600000001").await.unwrap();
    }

    // Phone 2 must still work — different rate limit key
    let result = svc.request_otp(&user_id, "+237600000002").await;
    assert!(
        result.is_ok(),
        "Different phone number must not be rate limited"
    );
}
