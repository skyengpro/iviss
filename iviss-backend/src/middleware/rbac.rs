use crate::app_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::{decode_access_token_rs256, extract_bearer_token};
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedAdmin {
    pub user_id: Uuid,
    pub role: String,
    pub organization_id: Option<Uuid>,
    pub email: String,
}

/// JWT middleware for web users (admin / manager / org_admin).
///
/// Validates JWT signature and expiry, then looks up the user's
/// `organization_id` from the database.
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

    // Look up the user's organization_id and email from the database
    let row = sqlx::query(
        "SELECT organization_id, email FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::database)?;

    let (org_id, email): (Option<Uuid>, Option<String>) = row
        .map(|r| (r.get("organization_id"), r.get("email")))
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let email = email.unwrap_or_default();

    tracing::info!(
        %method,
        %path,
        user_id = %claims.sub,
        role = %claims.role,
        org_id = ?org_id,
        "rbac: jwt verified"
    );

    request.extensions_mut().insert(AuthenticatedAdmin {
        user_id: claims.sub,
        role: claims.role.clone(),
        organization_id: org_id,
        email,
    });

    Ok(next.run(request).await)
}

/// Role guard — allows `admin` and `org_admin`.
///
/// Returns 403 if the user is neither an admin nor an org_admin.
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

    if user.role != "admin" && user.role != "org_admin" {
        tracing::warn!(
            %method,
            %path,
            user_id = %user.user_id,
            role = %user.role,
            "rbac: access denied — not an admin or org_admin"
        );
        return Err(AppError::forbidden("Admin access required"));
    }

    tracing::info!(%method, %path, user_id = %user.user_id, role = %user.role, "rbac: admin access granted");

    Ok(next.run(request).await)
}

/// Role guard — requires `org_admin` with a valid `organization_id`.
///
/// Returns 403 if the user is not an org_admin or has no organization.
pub async fn require_org_admin(request: Request, next: Next) -> Result<Response, AppError> {
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

    if user.role != "org_admin" {
        tracing::warn!(
            %method,
            %path,
            user_id = %user.user_id,
            role = %user.role,
            "rbac: access denied — not an org_admin"
        );
        return Err(AppError::forbidden("Org admin access required"));
    }

    if user.organization_id.is_none() {
        tracing::error!(
            %method,
            %path,
            user_id = %user.user_id,
            "rbac: org_admin has no organization_id"
        );
        return Err(AppError::forbidden(
            "Org admin must belong to an organization",
        ));
    }

    tracing::info!(%method, %path, user_id = %user.user_id, "rbac: org_admin access granted");

    Ok(next.run(request).await)
}
