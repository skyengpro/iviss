use crate::app_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::{decode_access_token_rs256, extract_bearer_token};
use crate::services::jwt_service::AccessTokenClaims;
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedAdmin {
    pub user_id: Uuid,
    pub role: String,
}

impl From<&AccessTokenClaims> for AuthenticatedAdmin {
    fn from(claims: &AccessTokenClaims) -> Self {
        Self {
            user_id: claims.sub,
            role: claims.role.clone(),
        }
    }
}
/// JWT middleware for web users (admin / manager).
///
/// Validates JWT signature, expiry, and checks Moka cache blacklist.
pub async fn require_auth_web(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    tracing::info!(%method, %path, "rbac: require_auth_web start");

    let token = extract_bearer_token(request.headers().get(AUTHORIZATION)).map_err(|err| {
        tracing::warn!(%method, %path, error = %err, "rbac: missing/invalid authorization header");
        err
    })?;

    let claims = decode_access_token_rs256(token, &state.jwt_public_key_pem).map_err(|err| {
        tracing::warn!(%method, %path, error = %err, "rbac: jwt verification failed");
        err
    })?;

    tracing::info!(
        %method,
        %path,
        user_id = %claims.sub,
        role = %claims.role,
        jti = %claims.jti,
        "rbac: jwt verified"
    );

    // Check Moka cache blacklist for admin tokens
    let is_blacklisted = state
        .app_cache
        .jti_blacklist
        .get(&claims.jti.to_string())
        .await
        .is_some();

    if is_blacklisted {
        tracing::warn!(
            %method,
            %path,
            user_id = %claims.sub,
            jti = %claims.jti,
            "rbac: rejected (token revoked)"
        );
        return Err(AppError::unauthorized("Token has been revoked"));
    }

    request
        .extensions_mut()
        .insert(AuthenticatedAdmin::from(&claims));

    Ok(next.run(request).await)
}

/// Role guard.
///
/// Returns 403 if the user is not an admin.
pub async fn require_admin(request: Request, next: Next) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let user = request
        .extensions()
        .get::<AuthenticatedAdmin>()
        .cloned()
        .ok_or_else(|| {
            tracing::error!(%method, %path, "rbac: AuthenticatedAdmin missing — require_auth_web must run first");
            AppError::internal_error("Authentication context missing")
        })?;

    if user.role != "admin" {
        tracing::warn!(
            %method,
            %path,
            user_id = %user.user_id,
            role = %user.role,
            "rbac: access denied — not an admin"
        );
        return Err(AppError::forbidden("Admin access required"));
    }

    tracing::info!(%method, %path, user_id = %user.user_id, "rbac: admin access granted");

    Ok(next.run(request).await)
}
