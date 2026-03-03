use crate::app_state::AppState;
use crate::errors::AppError;
use crate::queries::auth_queries;
use axum::extract::{Request, State};
use axum::http::header::{HeaderValue, AUTHORIZATION};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub jti: String,
    pub device_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl From<&JwtClaims> for AuthenticatedUser {
    fn from(claims: &JwtClaims) -> Self {
        Self {
            user_id: claims.sub,
        }
    }
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(request.headers().get(AUTHORIZATION))?;
    let claims = decode_jwt(token, &state.jwt_secret)?;

    let validation_context = auth_queries::get_auth_validation_context(
        &state.db,
        claims.sub,
        claims.device_id,
        &claims.jti,
    )
    .await?;

    if validation_context.is_blacklisted {
        return Err(AppError::unauthorized("Token has been revoked"));
    }

    match validation_context.user_status.as_deref() {
        Some(status) if is_user_status_allowed(status) => {}
        Some("SUSPENDED") => return Err(AppError::unauthorized("User account is suspended")),
        Some(_) => return Err(AppError::unauthorized("User account is not active")),
        None => return Err(AppError::unauthorized("User not found")),
    }

    if !validation_context.device_is_active {
        return Err(AppError::unauthorized(
            "Device is not active or not bound to user",
        ));
    }

    request
        .extensions_mut()
        .insert(AuthenticatedUser::from(&claims));

    Ok(next.run(request).await)
}

fn extract_bearer_token(header: Option<&HeaderValue>) -> Result<&str, AppError> {
    let auth_header = header
        .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("Invalid Authorization header encoding"))?;

    auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::unauthorized("Authorization header must use Bearer token"))
}

fn decode_jwt(token: &str, jwt_secret: &str) -> Result<JwtClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 0;

    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map(|token_data| token_data.claims)
    .map_err(|error| {
        tracing::warn!(error_kind = ?error.kind(), %error, "JWT decode failed");
        AppError::unauthorized("Invalid or expired token")
    })
}

fn is_user_status_allowed(status: &str) -> bool {
    status == "ACTIVE"
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use time::OffsetDateTime;

    fn build_claims(exp_offset_seconds: i64) -> JwtClaims {
        let exp = (OffsetDateTime::now_utc().unix_timestamp() + exp_offset_seconds) as usize;
        JwtClaims {
            sub: Uuid::new_v4(),
            exp,
            jti: Uuid::new_v4().to_string(),
            device_id: Uuid::new_v4(),
        }
    }

    fn sign_token(claims: &JwtClaims, secret: &str) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("failed to sign token")
    }

    #[test]
    fn extracts_bearer_token() {
        let header = HeaderValue::from_static("Bearer token-value");
        let token = extract_bearer_token(Some(&header)).expect("expected bearer token");
        assert_eq!(token, "token-value");
    }

    #[test]
    fn rejects_non_bearer_header() {
        let header = HeaderValue::from_static("Basic abc123");
        let result = extract_bearer_token(Some(&header));
        assert!(result.is_err());
    }

    #[test]
    fn decodes_valid_jwt() {
        let secret = "01234567890123456789012345678901";
        let claims = build_claims(300);
        let token = sign_token(&claims, secret);

        let decoded = decode_jwt(&token, secret).expect("expected valid token");
        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.device_id, claims.device_id);
        assert_eq!(decoded.jti, claims.jti);
    }

    #[test]
    fn rejects_expired_jwt() {
        let secret = "01234567890123456789012345678901";
        let claims = build_claims(-120);
        let token = sign_token(&claims, secret);

        let result = decode_jwt(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_suspended_user_status() {
        assert!(!is_user_status_allowed("SUSPENDED"));
    }

    #[test]
    fn allows_active_user_status() {
        assert!(is_user_status_allowed("ACTIVE"));
    }
}
