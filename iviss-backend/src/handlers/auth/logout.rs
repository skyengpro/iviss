use super::*;

/// Logout and invalidate session
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    params(LogoutRequestHeaders),
    responses(
        (status = 204, description = "Logout successful"),
        (status = 401, description = "Unauthorized - invalid or missing token", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "logoutUser",
    security(("bearer_auth" = []))
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<impl IntoResponse, AppError> {
    // Extract the authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?
        .to_str()
        .map_err(|_| AppError::unauthorized("Invalid Authorization header encoding"))?;

    // Parse Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::unauthorized("Authorization header must start with Bearer "))?;

    // Decode the token to get claims (JTI, user_id, exp)
    let claims = decode_access_token_rs256(token, &state.jwt_public_key_pem)?;

    // Calculate remaining TTL for the token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time error"))?
        .as_secs() as usize;

    let ttl = if claims.exp > now {
        (claims.exp - now) as u64
    } else {
        0
    };

    // Blacklist the JTI in PostgreSQL for persistence (prevents further use of this access token)
    if ttl > 0 {
        let expires_at = time::OffsetDateTime::now_utc() + time::Duration::seconds(ttl as i64);
        auth_queries::blacklist_jti_db(&state.db, &claims.jti.to_string(), claims.sub, expires_at)
            .await?;

        auth_queries::blacklist_jti_cache(&state.app_cache, &claims.jti.to_string()).await?;
    } else {
        tracing::warn!(
            target: "audit",
            event = "logout",
            user_id = %claims.sub,
            role = %claims.role,
            jti = %claims.jti,
            "Attempted to blacklist expired token"
        );
    }

    revoke_all_user_refresh_tokens(&state.db, claims.sub).await?;

    // Audit log
    tracing::info!(
        target: "audit",
        event = "logout",
        user_id = %claims.sub,
        role = %claims.role,
        jti = %claims.jti,
        "Admin logout executed"
    );

    // Return 204 No Content (idempotent - success even if token was already blacklisted)
    Ok(StatusCode::NO_CONTENT)
}

/// Revoke all refresh tokens for a user
async fn revoke_all_user_refresh_tokens(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE, revoked_at = NOW()
        WHERE user_id = $1
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::database)
}
