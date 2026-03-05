use crate::services::jwt_service::JwtService;
use uuid::Uuid;

fn make_jwt_service() -> JwtService {
    JwtService::new("test-secret-that-is-at-least-32-chars!!".to_string())
}

// ─────────────────────────────────────────
// Token generation
// ─────────────────────────────────────────

#[test]
fn test_issue_token_pair_returns_non_empty_tokens() {
    let svc = make_jwt_service();
    let pair = svc
        .issue_token_pair(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();

    assert!(!pair.access_token.is_empty());
    assert!(!pair.refresh_token.is_empty());
    assert!(!pair.refresh_token_jti.is_empty());
}

#[test]
fn test_access_and_refresh_tokens_are_different() {
    let svc = make_jwt_service();
    let pair = svc
        .issue_token_pair(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();

    assert_ne!(pair.access_token, pair.refresh_token);
}

#[test]
fn test_shift_expires_at_is_8h_from_now() {
    use time::OffsetDateTime;

    let svc = make_jwt_service();
    let pair = svc
        .issue_token_pair(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();

    let now = OffsetDateTime::now_utc().unix_timestamp() as usize;
    let expected_shift = now + (8 * 3600);

    // Allow 5s tolerance
    assert!(
        pair.shift_expires_at >= expected_shift - 5 && pair.shift_expires_at <= expected_shift + 5,
        "shift_expires_at must be ~8h from now"
    );
}

#[test]
fn test_two_token_pairs_have_different_jtis() {
    use crate::services::jwt_service::JwtClaims;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let svc = make_jwt_service();
    let secret = "test-secret-that-is-at-least-32-chars!!";

    let pair1 = svc
        .issue_token_pair(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();
    let pair2 = svc
        .issue_token_pair(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;

    let claims1 = decode::<JwtClaims>(
        &pair1.access_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .unwrap()
    .claims;

    let claims2 = decode::<JwtClaims>(
        &pair2.access_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .unwrap()
    .claims;

    assert_ne!(
        claims1.jti, claims2.jti,
        "Each token must have a unique jti"
    );
}

#[test]
fn test_token_contains_correct_sub_and_device_id() {
    use crate::services::jwt_service::JwtClaims;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let svc = make_jwt_service();
    let secret = "test-secret-that-is-at-least-32-chars!!";
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let pair = svc.issue_token_pair(user_id, device_id).unwrap();

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;

    let claims = decode::<JwtClaims>(
        &pair.access_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .unwrap()
    .claims;

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.device_id, device_id);
}
