use crate::app_cache::AppCache;
use crate::services::email_provider::MockEmailProvider;
use crate::services::otp_service::OtpService;
use crate::services::sms_provider::MockSmsProvider;
use std::sync::Arc;
use uuid::Uuid;

// ─────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────

async fn setup_otp_service() -> OtpService {
    let cache = Arc::new(AppCache::new());
    let sms_provider = Arc::new(MockSmsProvider);

    let email_provider = Arc::new(MockEmailProvider);

    OtpService::new(
        cache,
        sms_provider,
        email_provider,
        "test-pepper-that-is-at-least-32-chars!!".to_string(),
        false,
    )
}

// ─────────────────────────────────────────
// OTP generation & validation
// ─────────────────────────────────────────

#[tokio::test]
async fn test_request_otp_succeeds() {
    let _svc = setup_otp_service().await;
    let svc = setup_otp_service().await;

    let user_id = Uuid::new_v4();
    let result = svc.request_otp(&user_id, "+237600000000").await;
    assert!(result.is_ok(), "OTP request must succeed");
}

#[tokio::test]
async fn test_validate_otp_wrong_code_fails() {
    let svc = setup_otp_service().await;

    let user_id = Uuid::new_v4();
    svc.request_otp(&user_id, "+237600000000").await.unwrap();

    let result = svc.validate_otp(&user_id, "000000").await;
    assert!(result.is_err(), "Wrong OTP must fail");
}

#[tokio::test]
async fn test_validate_otp_no_key_fails() {
    let svc = setup_otp_service().await;

    // No OTP requested — key doesn't exist
    let result = svc.validate_otp(&Uuid::new_v4(), "123456").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expired or not found"));
}

// ─────────────────────────────────────────
// Rate limiting
// ─────────────────────────────────────────

#[tokio::test]
async fn test_rate_limit_blocks_after_3_requests() {
    let svc = setup_otp_service().await;

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
    let svc = setup_otp_service().await;

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
