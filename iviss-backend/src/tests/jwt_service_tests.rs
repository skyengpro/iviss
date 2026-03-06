use crate::dto::users::UserRole;
use crate::services::jwt_service::{AccessTokenClaims, JwtService, RefreshTokenClaims};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use uuid::Uuid;

// ─────────────────────────────────────────
// Test fixtures
// ─────────────────────────────────────────

// RSA private key for signing (test only)
const TEST_PRIVATE_KEY: &str = include_str!("fixtures/test_private_key.pem");

// RSA public key for verification (test only)
const TEST_PUBLIC_KEY: &str = include_str!("fixtures/test_public_key.pem");

fn make_jwt_service() -> JwtService {
    JwtService::new(TEST_PRIVATE_KEY).expect("Failed to create JwtService")
}

fn decode_access_claims(token: &str) -> AccessTokenClaims {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;

    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY.as_bytes()).unwrap(),
        &validation,
    )
    .expect("Failed to decode access token")
    .claims
}

fn decode_refresh_claims(token: &str) -> RefreshTokenClaims {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;

    decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY.as_bytes()).unwrap(),
        &validation,
    )
    .expect("Failed to decode refresh token")
    .claims
}

// ─────────────────────────────────────────
// issue_access_token
// ─────────────────────────────────────────

#[test]
fn test_access_token_contains_correct_sub_and_device_id() {
    let svc = make_jwt_service();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let token = svc
        .issue_access_token(user_id, device_id, UserRole::Agent)
        .unwrap();

    let claims = decode_access_claims(&token);
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.device_id, device_id);
}

#[test]
fn test_access_token_role_is_correct() {
    let svc = make_jwt_service();

    let token = svc
        .issue_access_token(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    let claims = decode_access_claims(&token);
    assert_eq!(claims.role, UserRole::Agent.as_str());
}

#[test]
fn test_access_token_has_unique_jti() {
    let svc = make_jwt_service();

    let token1 = svc
        .issue_access_token(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();
    let token2 = svc
        .issue_access_token(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    let claims1 = decode_access_claims(&token1);
    let claims2 = decode_access_claims(&token2);
    assert_ne!(
        claims1.jti, claims2.jti,
        "Each token must have a unique jti"
    );
}

#[test]
fn test_access_token_shift_expires_at_equals_exp() {
    let svc = make_jwt_service();

    let token = svc
        .issue_access_token(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    let claims = decode_access_claims(&token);
    assert_eq!(claims.shift_expires_at, claims.exp);
}

// ─────────────────────────────────────────
// issue_shift_token_pair
// ─────────────────────────────────────────

#[test]
fn test_shift_token_pair_returns_non_empty_tokens() {
    let svc = make_jwt_service();
    let pair = svc
        .issue_shift_token_pair(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    assert!(!pair.access_token.is_empty());
    assert!(!pair.refresh_token.is_empty());
}

#[test]
fn test_shift_token_access_and_refresh_are_different() {
    let svc = make_jwt_service();
    let pair = svc
        .issue_shift_token_pair(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    assert_ne!(pair.access_token, pair.refresh_token);
}

#[test]
fn test_shift_token_expires_at_is_8h_from_now() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let svc = make_jwt_service();
    let pair = svc
        .issue_shift_token_pair(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let expected = now + (8 * 3600);

    // Allow 5s tolerance
    assert!(
        pair.shift_expires_at >= expected - 5 && pair.shift_expires_at <= expected + 5,
        "shift_expires_at must be ~8h from now"
    );
}

#[test]
fn test_shift_token_sub_and_device_id_are_correct() {
    let svc = make_jwt_service();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let pair = svc
        .issue_shift_token_pair(user_id, device_id, UserRole::Agent)
        .unwrap();

    let claims = decode_access_claims(&pair.access_token);
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.device_id, device_id);
}

// ─────────────────────────────────────────
// issue_refresh_token
// ─────────────────────────────────────────

#[test]
fn test_refresh_token_sub_and_device_id_are_correct() {
    let svc = make_jwt_service();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let (token, _jti) = svc.issue_refresh_token(user_id, device_id).unwrap();

    let claims = decode_refresh_claims(&token);
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.device_id, device_id);
}

#[test]
fn test_refresh_token_jti_matches_returned_jti() {
    let svc = make_jwt_service();

    let (token, jti) = svc
        .issue_refresh_token(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();

    let claims = decode_refresh_claims(&token);
    assert_eq!(claims.jti, jti, "jti in claims must match returned jti");
}

#[test]
fn test_refresh_token_expiry_is_30_days() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let svc = make_jwt_service();
    let (token, _) = svc
        .issue_refresh_token(Uuid::new_v4(), Uuid::new_v4())
        .unwrap();

    let claims = decode_refresh_claims(&token);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let expected = now + (30 * 24 * 3600);

    // Allow 5s tolerance
    assert!(
        claims.exp >= expected - 5 && claims.exp <= expected + 5,
        "Refresh token must expire in ~30 days"
    );
}
