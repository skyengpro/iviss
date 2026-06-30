use crate::dto::users::UserRole;
use crate::services::jwt_service::{AccessTokenClaims, JwtService};
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
    validation.set_audience(&["iviss-backend"]);

    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY.as_bytes()).unwrap(),
        &validation,
    )
    .expect("Failed to decode access token")
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
fn test_access_token_contains_shift_bounds() {
    let svc = make_jwt_service();

    let token = svc
        .issue_access_token(Uuid::new_v4(), Uuid::new_v4(), UserRole::Agent)
        .unwrap();

    let claims = decode_access_claims(&token);
    // shift_start should be set to current time
    // shift_end should be shift_start + 8 hours
    assert!(claims.shift_end > claims.shift_start);
}

// ─────────────────────────────────────────
// issue_access_token_with_shift
// ─────────────────────────────────────────

#[test]
fn test_access_token_with_shift_custom_bounds() {
    let svc = make_jwt_service();
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let token = svc
        .issue_access_token_with_shift(user_id, device_id, UserRole::Manager, 1000, 5000)
        .unwrap();

    let claims = decode_access_claims(&token);
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.device_id, device_id);
    assert_eq!(claims.role, UserRole::Manager.as_str());
    assert_eq!(claims.shift_start, 1000);
    assert_eq!(claims.shift_end, 5000);
}

#[test]
fn test_access_token_with_shift_different_roles() {
    let svc = make_jwt_service();

    for role in [UserRole::Admin, UserRole::Agent, UserRole::Manager] {
        let token = svc
            .issue_access_token_with_shift(Uuid::new_v4(), Uuid::new_v4(), role, 0, 3600)
            .unwrap();

        let claims = decode_access_claims(&token);
        assert_eq!(claims.role, role.as_str());
    }
}
