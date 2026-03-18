use crate::app_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::{extract_bearer_token, decode_access_token_rs256, AuthenticatedUser};
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

/// JWT middleware for web users (admin / manager).
///
/// Validates JWT signature and expiry — NO device check, NO shift check.
/// Injects `AuthenticatedUser` into request extensions.
pub async fn require_auth_web(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    tracing::info!(%method, %path, "rbac: require_auth_web start");

    let token = extract_bearer_token(request.headers().get(AUTHORIZATION))
        .map_err(|err| {
            tracing::warn!(%method, %path, error = %err, "rbac: missing/invalid authorization header");
            err
        })?;

    let claims = decode_access_token_rs256(token, &state.jwt_public_key_pem)
        .map_err(|err| {
            tracing::warn!(%method, %path, error = %err, "rbac: jwt verification failed");
            err
        })?;

    tracing::info!(
        %method,
        %path,
        user_id = %claims.sub,
        role = %claims.role,
        "rbac: jwt verified"
    );

    request.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.sub,
        device_id: claims.device_id,
        role: claims.role.clone(),
    });

    Ok(next.run(request).await)
}

/// Role guard — must run AFTER `require_auth_web`.
///
/// Returns 403 if the user is not an admin.
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| {
            tracing::error!(%method, %path, "rbac: AuthenticatedUser missing — require_auth_web must run first");
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