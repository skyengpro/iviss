use crate::app_state::AppState;
use crate::errors::AppError;
use crate::services::jwt_service::AccessTokenClaims;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub role: String,
    #[allow(dead_code)]
    pub device_id: Uuid,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::unauthorized("Invalid Authorization scheme"))?;

        let decoding_key = DecodingKey::from_rsa_pem(state.jwt_public_key_pem.as_bytes())
            .map_err(|_| AppError::unauthorized("Invalid token"))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        let data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
            .map_err(|_| AppError::unauthorized("Invalid token"))?;

        Ok(Self {
            user_id: data.claims.sub,
            role: data.claims.role,
            device_id: data.claims.device_id,
        })
    }
}
