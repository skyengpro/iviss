use crate::app_state::AppState;
use crate::errors::AppError;
use crate::queries::auth;
use crate::services::auth::jwt::AccessTokenClaims;
use axum::extract::{Request, State};
use axum::http::header::{HeaderValue, AUTHORIZATION};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub role: String,
}

impl From<&AccessTokenClaims> for AuthenticatedUser {
    fn from(claims: &AccessTokenClaims) -> Self {
        Self {
            user_id: claims.sub,
            device_id: claims.device_id,
            role: claims.role.clone(),
        }
    }
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path();

    tracing::info!(%method, %path, "auth: start");

    let token = match extract_bearer_token(request.headers().get(AUTHORIZATION)) {
        Ok(token) => {
            tracing::info!(%method, %path, "auth: bearer token present");
            token
        }
        Err(err) => {
            tracing::warn!(%method, %path, error = %err, "auth: missing/invalid authorization header");
            return Err(err);
        }
    };

    let claims = match decode_access_token_rs256(token, &state.jwt_public_key_pem) {
        Ok(claims) => {
            tracing::info!(
                %method,
                %path,
                user_id = %claims.sub,
                device_id = %claims.device_id,
                role = %claims.role,
                "auth: jwt verified (signature + exp)"
            );
            claims
        }
        Err(err) => {
            tracing::warn!(%method, %path, error = %err, "auth: jwt verification failed");
            return Err(err);
        }
    };

    let now: usize = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time before UNIX_EPOCH"))?
        .as_secs()
        .try_into()
        .unwrap_or(0usize);

    if now > claims.shift_end {
        return Err(crate::handlers::auth::on_shift_ended(&state.db, claims.device_id).await);
    }

    let validation_context = auth::get_auth_validation_context(
        &state.db,
        claims.sub,
        claims.device_id,
        &claims.jti.to_string(),
    )
    .await?;

    tracing::info!(
        %method,
        %path,
        user_id = %claims.sub,
        device_id = %claims.device_id,
        jti = %claims.jti,
        is_blacklisted = validation_context.is_blacklisted,
        user_status = validation_context.user_status.as_deref().unwrap_or("<none>"),
        device_is_active = validation_context.device_is_active,
        "auth: db validation context loaded"
    );

    if validation_context.is_blacklisted {
        tracing::warn!(
            %method,
            %path,
            user_id = %claims.sub,
            jti = %claims.jti,
            "auth: rejected (token revoked)"
        );
        return Err(AppError::unauthorized("Token has been revoked"));
    }

    match validation_context.user_status.as_deref() {
        Some(status) if is_user_status_allowed(status) => {
            tracing::info!(%method, %path, user_id = %claims.sub, %status, "auth: user status allowed");
        }
        Some("SUSPENDED") => {
            tracing::warn!(%method, %path, user_id = %claims.sub, "auth: rejected (user suspended)");
            return Err(AppError::unauthorized("User account is suspended"));
        }
        Some(status) => {
            tracing::warn!(%method, %path, user_id = %claims.sub, %status, "auth: rejected (user not active)");
            return Err(AppError::unauthorized("User account is not active"));
        }
        None => {
            tracing::warn!(%method, %path, user_id = %claims.sub, "auth: rejected (user not found)");
            return Err(AppError::unauthorized("User not found"));
        }
    }

    // Device check: Agents MUST have an active and bound device.
    // Admin/Manager do not use physical devices for web login.
    if claims.role != "admin" && claims.role != "manager" && !validation_context.device_is_active {
        tracing::warn!(
            %method,
            %path,
            user_id = %claims.sub,
            device_id = %claims.device_id,
            role = %claims.role,
            "auth: rejected (device not active or not bound)"
        );
        return Err(AppError::unauthorized(
            "Device is not active or not bound to user",
        ));
    }

    tracing::info!(
        %method,
        %path,
        user_id = %claims.sub,
        device_id = %claims.device_id,
        role = %claims.role,
        "auth: accepted"
    );

    request
        .extensions_mut()
        .insert(AuthenticatedUser::from(&claims));

    Ok(next.run(request).await)
}

pub fn extract_bearer_token(header: Option<&HeaderValue>) -> Result<&str, AppError> {
    let auth_header = header
        .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("Invalid Authorization header encoding"))?;

    let mut parts = auth_header.split_whitespace();
    let scheme = parts
        .next()
        .ok_or_else(|| AppError::unauthorized("Invalid Authorization header"))?;
    let token = parts
        .next()
        .ok_or_else(|| AppError::unauthorized("Missing bearer token"))?;

    if !scheme.eq_ignore_ascii_case("bearer") {
        return Err(AppError::unauthorized(
            "Authorization header must use Bearer token",
        ));
    }

    if parts.next().is_some() {
        return Err(AppError::unauthorized("Invalid Authorization header"));
    }

    Ok(token)
}

pub fn decode_access_token_rs256(
    token: &str,
    jwt_public_key_pem: &str,
) -> Result<AccessTokenClaims, AppError> {
    let cleaned = jwt_public_key_pem.trim_matches('"').trim();

    // Try decoding as Base64 first, if it doesn't look like a standard PEM
    let raw_pem = if !cleaned.starts_with("-----") {
        match base64::Engine::decode(
            &base64::prelude::BASE64_STANDARD,
            cleaned.replace("\\n", "").replace("\n", "").trim(),
        ) {
            Ok(decoded) => String::from_utf8(decoded).unwrap_or_else(|_| cleaned.to_string()),
            Err(_) => cleaned.to_string(),
        }
    } else {
        cleaned.to_string()
    };

    let cleaned_pem = raw_pem.replace("\\n", "\n");
    let decoding_key = DecodingKey::from_rsa_pem(cleaned_pem.as_bytes()).map_err(|e| {
        tracing::error!(error = %e, "Failed to parse JWT RSA public key PEM");
        AppError::internal_error("JWT verification is misconfigured")
    })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.set_audience(&["iviss-backend"]);
    validation.leeway = 0;

    decode::<AccessTokenClaims>(token, &decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|error| {
            tracing::warn!(error_kind = ?error.kind(), %error, "JWT decode failed");

            if error.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                return AppError::unauthorized("Token expired");
            }

            AppError::unauthorized("Invalid token")
        })
}

fn is_user_status_allowed(status: &str) -> bool {
    status == "ACTIVE"
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_is_user_status_allowed() {
        assert!(is_user_status_allowed("ACTIVE"));
        assert!(!is_user_status_allowed("SUSPENDED"));
        assert!(!is_user_status_allowed("PENDING_ACTIVATION"));
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let result = extract_bearer_token(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let header = HeaderValue::from_static("Bearer tok extra");
        let result = extract_bearer_token(Some(&header));
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_pem() {
        let result = decode_access_token_rs256("token", "not a pem");
        assert!(result.is_err());
    }
}
