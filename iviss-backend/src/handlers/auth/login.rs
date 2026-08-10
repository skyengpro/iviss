use super::*;

/// Login with email and password (admin / manager only)
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "loginUser"
)]
#[instrument(name = "auth.login", skip(state, payload), fields(email = %payload.email))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    metrics::counter!("iviss_auth_attempts_total", "method" => "login").increment(1);

    if payload.email.trim().is_empty() || payload.password.trim().is_empty() {
        metrics::counter!("iviss_auth_failures_total", "reason" => "empty_credentials")
            .increment(1);
        return Err(AppError::bad_request("Email and password are required"));
    }

    let rate_limit_key = format!("login_attempts:{}", payload.email.to_lowercase().trim());
    let current_attempts = state
        .app_cache
        .rate_limit
        .get(&rate_limit_key)
        .await
        .unwrap_or(0);

    if current_attempts >= 5 {
        tracing::warn!(email = %payload.email, "login: rate limited");
        metrics::counter!("iviss_auth_failures_total", "reason" => "rate_limited").increment(1);
        return Err(AppError::unauthorized(
            "Too many login attempts. Please try again later.",
        ));
    }

    state
        .app_cache
        .rate_limit
        .insert(rate_limit_key.clone(), current_attempts + 1)
        .await;

    let user = auth::find_admin_by_identity(&state.db, &payload.email)
        .await?
        .ok_or_else(|| {
            metrics::counter!("iviss_auth_failures_total", "reason" => "user_not_found")
                .increment(1);
            AppError::unauthorized("Invalid credentials")
        })?;

    if user.status != UserStatus::Active && !user.must_change_password {
        tracing::warn!(
            email = %payload.email,
            status = %user.status.as_str(),
            "login: rejected — account not active"
        );
        metrics::counter!("iviss_auth_failures_total", "reason" => "account_not_active")
            .increment(1);
        return Err(AppError::unauthorized("Account is not active"));
    }

    // Verify password
    let password_hash = user.password_hash.clone();
    let password_input = payload.password.clone();
    let matches = crate::utils::password::verify_password(&password_input, &password_hash)
        .await
        .map_err(|_| AppError::unauthorized("Invalid credentials"))?;

    if !matches {
        tracing::warn!(email = %payload.email, "login: rejected — wrong password");
        metrics::counter!("iviss_auth_failures_total", "reason" => "wrong_password").increment(1);
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    //    Issue access token
    //    Admins have no device
    if user.role != UserRole::Admin
        && user.role != UserRole::Manager
        && user.role != UserRole::OrgAdmin
    {
        metrics::counter!("iviss_auth_failures_total", "reason" => "invalid_role").increment(1);
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time error"))?
        .as_secs();

    // shift_end = 24 hours from now for web sessions
    let shift_start = now as usize;
    let shift_end = (now + 86_400) as usize;

    let jwt_svc = &state.jwt_svc;

    let access_token = jwt_svc
        .issue_access_token_with_shift(user.id, Uuid::nil(), user.role, shift_start, shift_end)
        .map_err(AppError::Internal)?;

    // Generate refresh token
    let mut raw_token = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut raw_token);
    let refresh_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_token);

    //    Store refresh token hash in DB
    //    device_id = Uuid::nil() for admin (no physical device)
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(refresh_token.as_bytes());
        format!("{digest:x}")
    };

    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    auth::insert_web_refresh_token(&state.db, &token_hash, user.id, expires_at).await?;

    // Build user profile
    let user_profile = UserProfile {
        id: user.id,
        username: user.username.clone(),
        name: user.full_name.clone(),
        email: Some(user.email.clone()),
        role: user.role,
        organization_id: user.organization_id,
        organization: None,
        badge_id: None,
        phone_number: Some(user.phone_number.clone()),
        avatar_initials: None,
        status: UserStatus::Active,
        session_status: None,
        last_revoked_at: None,
        is_active: true,
    };

    tracing::info!(
        user_id = %user.id,
        email = %user.email,
        role = %user.role.as_str(),
        "login: success"
    );

    state.app_cache.rate_limit.invalidate(&rate_limit_key).await;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            access_token,
            refresh_token,
            user: user_profile,
            must_change_password: user.must_change_password,
        }),
    ))
}
