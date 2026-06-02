use crate::app_state::AppState;

use crate::dto::auth::{
    ActivateRequest, ActivateResponse, AuthResponse, ChangePasswordRequest, ChangePasswordResponse,
    LoginRequest, LogoutRequestHeaders, RefreshRequest, RequestDailyLoginRequest,
    RequestDailyLoginResponse, VerifyDailyLoginRequest, VerifyDailyLoginResponse,
};
use crate::middleware::auth::decode_access_token_rs256;
use axum::extract::{Extension, State};
use axum::http::header::AUTHORIZATION;
use axum::{http::StatusCode, response::IntoResponse, Json};
use base64::Engine;

use crate::dto::users::{UserProfile, UserRole, UserStatus};
use crate::errors::AppError;
use crate::middleware::rbac::AuthenticatedAdmin;
use crate::queries::auth_queries;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

const SHIFT_TTL: Duration = Duration::from_secs(8 * 60 * 60);

/// Logic to execute when a shift has ended.
/// Marks the device as inactive and returns an unauthorized error.
pub async fn on_shift_ended(pool: &sqlx::PgPool, device_id: Uuid) -> AppError {
    tracing::warn!(%device_id, "shift: ended logic triggered");

    if let Err(err) = crate::queries::auth_queries::mark_device_inactive(pool, device_id).await {
        tracing::error!(%device_id, error = %err, "shift: failed to mark device inactive");
    }

    AppError::unauthorized("Shift ended")
}

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
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.email.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::bad_request("Email and password are required"));
    }

    let user = auth_queries::find_admin_by_identity(&state.db, &payload.email)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid credentials"))?;

    if user.status != UserStatus::Active && !user.must_change_password {
        tracing::warn!(
            email = %payload.email,
            status = %user.status.as_str(),
            "login: rejected — account not active"
        );
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
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    //    Issue access token
    //    Admins have no device
    if user.role != UserRole::Admin
        && user.role != UserRole::Manager
        && user.role != UserRole::OrgAdmin
    {
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

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&token_hash)
    .bind(user.id)
    .bind(Option::<Uuid>::None) // No device for admin login
    .bind(expires_at)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

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

/// Activate an agent account by validating OTP and registering device public key
#[utoipa::path(
    post,
    path = "/api/v1/auth/activate",
    request_body = ActivateRequest,
    responses(
        (status = 200, description = "Activation successful", body = ActivateResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
        (status = 404, description = "User not found", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "activateDevice"
)]
pub async fn activate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivateRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.badge_id.trim().is_empty() {
        return Err(AppError::BadRequest("badgeId is required".into()));
    }
    if payload.activation_code.trim().is_empty() {
        return Err(AppError::BadRequest("activationCode is required".into()));
    }

    base64::engine::general_purpose::STANDARD
        .decode(payload.public_key_base64.as_bytes())
        .map_err(|_| AppError::BadRequest("publicKeyBase64 must be valid Base64".into()))?;

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    let user_row = sqlx::query(
        r#"
        SELECT id,
               role,
               status::TEXT AS status
        FROM users
        WHERE badge_id = $1
        AND deleted_at IS NULL
        "#,
    )
    .bind(&payload.badge_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let user_id: Uuid = user_row.get("id");
    let user_role: UserRole = user_row.get("role");
    let user_status: String = user_row.get("status");

    if user_role != UserRole::Agent {
        return Err(AppError::BadRequest(
            "Activation is only available for agents".into(),
        ));
    }
    if user_status != "PENDING_ACTIVATION" {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation - current status: {user_status}"
        )));
    }

    let otp_svc = &state.otp_svc;
    otp_svc
        .validate_otp(&user_id, &payload.activation_code)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time before UNIX_EPOCH"))?
        .as_secs();

    let shift_start: i64 = now.try_into().unwrap_or(0i64);
    let shift_end: i64 = now
        .saturating_add(SHIFT_TTL.as_secs())
        .try_into()
        .unwrap_or(0i64);

    sqlx::query(
        r#"
        UPDATE users
        SET status = 'ACTIVE'::user_status
        WHERE id = $1
        AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    sqlx::query(
        r#"
        INSERT INTO devices (id, user_id, public_key, status, metadata)
        VALUES (
            $1,
            $2,
            $3,
            'ACTIVE'::device_status,
            jsonb_build_object('shift_start', $4, 'shift_end', $5)
        )
        ON CONFLICT (id)
        DO UPDATE SET
            user_id = EXCLUDED.user_id,
            public_key = EXCLUDED.public_key,
            status = 'ACTIVE'::device_status,
            metadata = EXCLUDED.metadata,
            revoked_at = NULL
        "#,
    )
    .bind(payload.device_id)
    .bind(user_id)
    .bind(&payload.public_key_base64)
    .bind(shift_start)
    .bind(shift_end)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    let refresh_token = {
        let mut raw = [0u8; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut raw);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
    };
    let refresh_token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(refresh_token.as_bytes());
        format!("{digest:x}")
    };
    let refresh_expires_at = OffsetDateTime::now_utc() + time::Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&refresh_token_hash)
    .bind(user_id)
    .bind(payload.device_id)
    .bind(refresh_expires_at)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    let user = crate::queries::user_queries::get_user_by_id(&state.db, user_id).await?;

    let jwt_svc = &state.jwt_svc;
    let access_token = jwt_svc
        .issue_access_token_with_shift(
            user_id,
            payload.device_id,
            user.role,
            shift_start.try_into().unwrap_or(0usize),
            shift_end.try_into().unwrap_or(0usize),
        )
        .map_err(AppError::Internal)?;

    Ok((
        StatusCode::OK,
        Json(ActivateResponse {
            access_token,
            refresh_token,
            user,
        }),
    ))
}

// Request a daily OTP login code
#[utoipa::path(
    post,
    path = "/api/v1/auth/request-daily-login",
    request_body = RequestDailyLoginRequest,
    responses(
        (status = 201, description = "OTP sent successfully", body = RequestDailyLoginResponse),
        (status = 400, description = "Bad request — missing or invalid fields", body = AppErrorResponse),
        (status = 401, description = "User or device is suspended", body = AppErrorResponse),
        (status = 404, description = "User or device not found", body = AppErrorResponse),
        (status = 429, description = "Too many OTP requests", body = AppErrorResponse),
    ),
    tag = "auth",
    operation_id = "requestDailyLogin"
)]
pub async fn request_daily_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestDailyLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.badge_id.trim().is_empty() {
        return Err(AppError::bad_request("badgeId is required"));
    }
    let user = auth_queries::get_user_by_badge(&state.db, &payload.badge_id).await?;

    // Only agents use the daily OTP flow
    if user.role != "agent" {
        return Err(AppError::unauthorized(
            "Daily login is only available for agents",
        ));
    }

    if user.status == "SUSPENDED" {
        return Err(AppError::unauthorized(
            "Account suspended — contact your administrator",
        ));
    }

    // Enforce shift hours per organization (Cameroon local time UTC+1)
    // Agents must belong to an organization
    let user_org_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT organization_id
        FROM users
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::database)?
    .flatten();

    let org_id = user_org_id.ok_or_else(|| {
        AppError::forbidden("Agent must belong to an organization to request daily login")
    })?;

    let (shift_start_hour, shift_end_hour) =
        crate::queries::organization_queries::get_organization_work_time_cached(
            &state.db,
            &state.app_cache,
            org_id,
        )
        .await?;

    let local_offset = time::UtcOffset::from_hms(1, 0, 0).unwrap_or(time::UtcOffset::UTC);
    let now_local = time::OffsetDateTime::now_utc().to_offset(local_offset);
    let current_minute_of_day = (now_local.hour() as u32) * 60 + (now_local.minute() as u32);

    // Shift window: shift_start_hour/shift_end_hour are stored as minutes since midnight
    // (inclusive start, exclusive end)
    if current_minute_of_day < shift_start_hour || current_minute_of_day >= shift_end_hour {
        return Err(AppError::unauthorized(format!(
            "Outside shift hours — login is available from {} to {} local time",
            shift_start_hour, shift_end_hour
        )));
    }

    let device_opt =
        auth_queries::get_device_by_user_optional(&state.db, payload.device_id, user.id).await?;

    let device = device_opt.ok_or_else(|| {
        AppError::NotFound("Device is not registered. Please re-activate.".into())
    })?;

    if device.status == "SUSPENDED" {
        return Err(AppError::unauthorized(
            "Device suspended — contact your administrator",
        ));
    }

    // Check for administrative termination cooldown (Ariel's feedback)
    if let Some(revoked_at) = device.revoked_at {
        // Assume UTC for the stored TIMESTAMP (project convention)
        let local_offset = time::UtcOffset::from_hms(1, 0, 0).unwrap_or(time::UtcOffset::UTC);
        let revoked_local = revoked_at.to_offset(local_offset);
        let now = OffsetDateTime::now_utc().to_offset(local_offset);

        if revoked_local.date() == now.date() {
            return Err(AppError::Forbidden(
                "Session terminated by administrator. Please wait until your next shift (tomorrow at 8:00 AM) to request a new code.".into()
            ));
        }
    }

    let otp_svc = &state.otp_svc;

    // Determine contact (email or phone) based on AppState setting
    let contact = if state.otp_via_email {
        // Fetch full profile to obtain email if configured to use email
        let profile = crate::queries::user_queries::get_user_by_id(&state.db, user.id).await?;
        profile
            .email
            .clone()
            .unwrap_or_else(|| profile.phone_number.clone().unwrap_or_default())
    } else {
        user.phone_number.clone()
    };

    otp_svc.request_otp(&user.id, &contact).await?;

    tracing::info!(
        target: "daily_login",
        user_id = %user.id,
        device_id = %payload.device_id,
        "Daily login OTP requested"
    );

    Ok((
        StatusCode::CREATED,
        Json(RequestDailyLoginResponse {
            message: "OTP sent successfully".into(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-daily-login",
    request_body = VerifyDailyLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = VerifyDailyLoginResponse),
        (status = 400, description = "Bad request — missing or invalid fields", body = AppErrorResponse),
        (status = 401, description = "Invalid OTP, expired, or device suspended", body = AppErrorResponse),
        (status = 404, description = "User or device not found", body = AppErrorResponse),
    ),
    tag = "auth",
    operation_id = "verifyDailyLogin"
)]
pub async fn verify_daily_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyDailyLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.badge_id.trim().is_empty() {
        return Err(AppError::bad_request("badgeId is required"));
    }
    if payload.activation_code.trim().is_empty() {
        return Err(AppError::bad_request("activationCode is required"));
    }

    let row = sqlx::query(
        r#"
        SELECT
            u.id              AS user_id,
            u.role            AS user_role,
            u.status          AS user_status,
            COALESCE(d.status::TEXT, 'INACTIVE') AS device_status
        FROM users u
        LEFT JOIN devices d
            ON d.user_id = u.id
           AND d.id      = $2
        WHERE u.badge_id    = $1
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(&payload.badge_id)
    .bind(payload.device_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::not_found("User or device not found"))?;

    let user_id: Uuid = row.get("user_id");
    let user_role: UserRole = row.get("user_role");
    let user_status: UserStatus = row.get("user_status");
    let device_status: String = row.get("device_status");

    // ── Status checks
    if user_role != UserRole::Agent {
        return Err(AppError::unauthorized(
            "Daily login is only available for agents",
        ));
    }

    if user_status != UserStatus::Active {
        return Err(AppError::unauthorized(format!(
            "User account is {}",
            user_status.as_str()
        )));
    }

    // Daily login doesn't REQUIRE the device to be registered yet
    // but if it IS registered, it must not be suspended/revoked
    if device_status == "SUSPENDED" || device_status == "REVOKED" {
        return Err(AppError::unauthorized(format!(
            "Device status: {}",
            device_status.to_lowercase()
        )));
    }

    // ── Validate OTP
    let otp_svc = &state.otp_svc;
    otp_svc
        .validate_otp(&user_id, &payload.activation_code)
        .await?;

    // ── Compute static shift bounds (UTC+1 local time)
    let localt_time_offset = time::UtcOffset::from_hms(1, 0, 0)
        .map_err(|_| AppError::internal_error("Failed to build UTC+1 offset"))?;

    let today_local = time::OffsetDateTime::now_utc()
        .to_offset(localt_time_offset)
        .date();

    // Determine shift hours from the user's organization configuration
    let user_org_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT organization_id
        FROM users
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::database)?
    .flatten();

    let org_id = user_org_id
        .ok_or_else(|| AppError::forbidden("Agent must belong to an organization to login"))?;

    let (shift_start_minutes, shift_end_minutes) =
        crate::queries::organization_queries::get_organization_work_time_cached(
            &state.db,
            &state.app_cache,
            org_id,
        )
        .await?;

    let shift_start_hour = (shift_start_minutes / 60) as u8;
    let shift_start_minute = (shift_start_minutes % 60) as u8;
    let shift_end_hour = (shift_end_minutes / 60) as u8;
    let shift_end_minute = (shift_end_minutes % 60) as u8;

    let shift_start_time = time::Time::from_hms(shift_start_hour, shift_start_minute, 0)
        .map_err(|_| AppError::internal_error("Invalid shift_start_hour in organization"))?;

    let shift_end_time = time::Time::from_hms(shift_end_hour, shift_end_minute, 0)
        .map_err(|_| AppError::internal_error("Invalid shift_end_hour in organization"))?;

    let shift_start: i64 =
        time::OffsetDateTime::new_in_offset(today_local, shift_start_time, localt_time_offset)
            .unix_timestamp();

    let shift_end: i64 =
        time::OffsetDateTime::new_in_offset(today_local, shift_end_time, localt_time_offset)
            .unix_timestamp();

    // ── Issue access token (15 min, carries today's static shift bounds) ──────
    let jwt_svc = &state.jwt_svc;

    let access_token = jwt_svc
        .issue_access_token_with_shift(
            user_id,
            payload.device_id,
            user_role,
            shift_start.try_into().unwrap_or(0usize),
            shift_end.try_into().unwrap_or(0usize),
        )
        .map_err(AppError::Internal)?;

    let device_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM devices
            WHERE id = $1
              AND user_id = $2
              AND suspended_at IS NULL
        )
        "#,
    )
    .bind(payload.device_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    if !device_exists {
        return Err(AppError::NotFound(
            "Device is not registered. Please re-activate.".into(),
        ));
    }

    // ── Check if a valid refresh token already exists for this device
    let has_valid_refresh: bool = if device_exists {
        auth_queries::has_valid_refresh_token(&state.db, payload.device_id).await?
    } else {
        false
    };

    // ── Conditionally build new refresh token
    let new_refresh: Option<(String, String, time::OffsetDateTime)> =
        if device_exists && !has_valid_refresh {
            let raw = {
                let mut bytes = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut bytes);
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
            };
            let hash = {
                use sha2::Digest;
                let digest = sha2::Sha256::digest(raw.as_bytes());
                format!("{digest:x}")
            };
            let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);
            Some((raw, hash, expires_at))
        } else {
            None
        };

    // ── Single CTE: optionally insert refresh token + activate device
    match &new_refresh {
        Some((_, hash, expires_at)) => {
            sqlx::query(
                r#"
                WITH insert_refresh AS (
                    INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at)
                    VALUES ($2, $3, $1, $4)
                )
                UPDATE devices
                SET    status       = 'ACTIVE'::device_status,
                       metadata     = jsonb_build_object('shift_start', $5, 'shift_end', $6),
                       last_seen_at = NOW()
                WHERE  id = $1
                "#,
            )
            .bind(payload.device_id) // $1
            .bind(hash) // $2
            .bind(user_id) // $3
            .bind(expires_at) // $4
            .bind(shift_start) // $5
            .bind(shift_end) // $6
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
        }

        None => {
            auth_queries::mark_device_active(&state.db, payload.device_id, shift_start, shift_end)
                .await?;
        }
    }

    tracing::info!(
        target: "daily_login",
        user_id            = %user_id,
        device_id          = %payload.device_id,
        shift_start,
        shift_end,
        new_refresh_issued = new_refresh.is_some(),
        "Daily login verified — shift started"
    );

    Ok((
        StatusCode::OK,
        Json(VerifyDailyLoginResponse {
            access_token,
            refresh_token: new_refresh.map(|(plain, _, _)| plain),
            shift_end,
        }),
    ))
}

// ─────────────────────────────────────────────────────────────
//  Token Refresh — Challenge-Response with Device Signature
// ─────────────────────────────────────────────────────────────

const NONCE_TTL_SECS: u64 = 60;

// RefreshRequest is already defined at line 171

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshChallengeResponse {
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRefreshRequest {
    pub refresh_token: String,
    pub device_id: Uuid,
    pub signed_nonce: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRefreshResponse {
    pub access_token: String,
}

/// Step 1 of the challenge-response refresh flow.
///
/// Validates the refresh token, generates a nonce, stores it in Moka cache,
/// and returns it to the client for signing.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Challenge nonce issued", body = RefreshChallengeResponse),
        (status = 401, description = "Invalid or expired refresh token", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "requestRefresh"
)]
pub async fn request_refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.refresh_token.trim().is_empty() {
        return Err(AppError::bad_request("refresh_token is required"));
    }

    match payload.device_id {
        Some(_) => request_refresh_agent(state, payload).await,
        None => request_refresh_admin(state, payload.refresh_token).await,
    }
}

async fn request_refresh_agent(
    state: Arc<AppState>,
    payload: RefreshRequest,
) -> Result<axum::response::Response, AppError> {
    let device_id = payload
        .device_id
        .ok_or(AppError::bad_request("device_id is required"))?;
    // Hash the incoming refresh token to match against DB
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(payload.refresh_token.as_bytes());
        format!("{digest:x}")
    };

    // Validate refresh token exists, is not revoked, and not expired
    let token_row = sqlx::query(
        r#"
        SELECT user_id, device_id
        FROM refresh_tokens
        WHERE token_hash = $1
          AND device_id = $2
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .bind(device_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    if token_row.is_none() {
        return Err(AppError::Unauthorized(
            "Invalid or expired refresh token".into(),
        ));
    }

    // Generate a random 32-byte nonce
    let nonce = {
        let mut raw = [0u8; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut raw);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
    };

    // Store nonce in Moka cache with device_id as key, TTL 60s (handled automatically)
    state
        .app_cache
        .refresh_nonce
        .insert(device_id, nonce.clone())
        .await;

    tracing::info!(
        %device_id,
        "Refresh nonce issued (TTL={}s)",
        NONCE_TTL_SECS
    );

    Ok((
        axum::http::StatusCode::OK,
        Json(RefreshChallengeResponse { nonce }),
    )
        .into_response())
}

async fn request_refresh_admin(
    state: Arc<AppState>,
    refresh_token: String,
) -> Result<axum::response::Response, AppError> {
    tracing::warn!("admin refresh: attempt received");

    // Hash the refresh token
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(refresh_token.as_bytes());
        format!("{digest:x}")
    };

    // Validate refresh token — device_id must be NULL (admin token)
    let row = sqlx::query(
        r#"
        SELECT
            rt.user_id,
            role,
            status
        FROM refresh_tokens rt
        JOIN users u ON u.id = rt.user_id
        WHERE rt.token_hash = $1
          AND rt.device_id IS NULL
          AND rt.revoked = FALSE
          AND rt.expires_at > NOW()
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "admin refresh: database error during token lookup");
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        tracing::warn!("admin refresh: FAILED — refresh token not found, revoked, or expired");
        AppError::Unauthorized("Invalid or expired refresh token".into())
    })?;

    let user_id: Uuid = row.get("user_id");
    let role: UserRole = row.get("role");
    let status: UserStatus = row.get("status");

    // Check account still active
    if status != UserStatus::Active {
        tracing::warn!(%user_id, status = %status.as_str(), "admin refresh: FAILED — account not active");
        return Err(AppError::Unauthorized("Account is not active".into()));
    }

    if !matches!(
        role,
        UserRole::Admin | UserRole::Manager | UserRole::OrgAdmin
    ) {
        tracing::warn!(%user_id, role = %role.as_str(), "admin refresh: FAILED — role not authorized for web refresh");
        return Err(AppError::forbidden("Not authorized for web refresh"));
    }

    // Issue new access token
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time error"))?
        .as_secs();

    let shift_start = now as usize;
    let shift_end = (now + 86_400) as usize;

    let jwt_svc = &state.jwt_svc;

    let access_token = jwt_svc
        .issue_access_token_with_shift(user_id, Uuid::nil(), role, shift_start, shift_end)
        .map_err(|e| {
            tracing::warn!(%user_id, error = %e, "admin refresh: FAILED — could not issue access token");
            AppError::Internal(e)
        })?;

    tracing::warn!(
        %user_id,
        role = %role.as_str(),
        "admin refresh: SUCCESS — new access token issued"
    );

    Ok((
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "accessToken": access_token })),
    )
        .into_response())
}

/// Step 2 of the challenge-response refresh flow.
///
/// Verifies the signed nonce against the device's registered public key,
/// then issues a new access token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh/verify",
    request_body = VerifyRefreshRequest,
    responses(
        (status = 200, description = "New access token issued", body = VerifyRefreshResponse),
        (status = 401, description = "Signature verification failed", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "verifyRefresh"
)]
pub async fn verify_refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyRefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::warn!(device_id = %payload.device_id, "--- [BACKEND] verify_refresh: Processing start ---");

    // Retrieve and consume the nonce from Moka cache (one-time use)
    let stored_nonce: Option<String> = {
        // Get the nonce from cache
        let nonce = state.app_cache.refresh_nonce.get(&payload.device_id).await;

        // Immediately invalidate to ensure single-use (prevent replay)
        if nonce.is_some() {
            state
                .app_cache
                .refresh_nonce
                .invalidate(&payload.device_id)
                .await;
        }

        nonce
    };

    let expected_nonce = stored_nonce.ok_or_else(|| {
        tracing::warn!(device_id = %payload.device_id, "Verification failed: Nonce not found or expired");
        AppError::Unauthorized("Nonce expired or not found — request a new challenge".into())
    })?;

    tracing::warn!(nonce = %expected_nonce, "Step 1: Nonce retrieved and consumed from Moka cache");

    // 2. Validate the refresh token
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(payload.refresh_token.as_bytes());
        format!("{digest:x}")
    };

    let token_row = sqlx::query(
        r#"
        SELECT user_id
        FROM refresh_tokens
        WHERE token_hash = $1
          AND device_id = $2
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .bind(payload.device_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| {
        tracing::warn!(device_id = %payload.device_id, "Verification failed: Invalid or expired refresh token");
        AppError::Unauthorized("Invalid or expired refresh token".into())
    })?;

    let user_id: Uuid = token_row.get("user_id");
    tracing::warn!(user_id = %user_id, "Step 2: Refresh token validated in database");

    // 3. Fetch the device's public key & shift metadata
    let device_row = sqlx::query(
        r#"
        SELECT public_key, metadata
        FROM devices
        WHERE id = $1
          AND user_id = $2
          AND status = 'ACTIVE'::device_status
        "#,
    )
    .bind(payload.device_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| {
        tracing::warn!(device_id = %payload.device_id, "Verification failed: Device not found or revoked");
        AppError::Unauthorized("Device not found or revoked".into())
    })?;

    tracing::warn!("Step 3: Device public key and shift metadata fetched");

    let public_key_b64: String = device_row.get("public_key");
    let metadata: serde_json::Value = device_row.get("metadata");

    let shift_start = metadata
        .get("shift_start")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::internal_error("Device shift_start is missing"))?;
    let shift_end = metadata
        .get("shift_end")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::internal_error("Device shift_end is missing"))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::internal_error("System time before UNIX_EPOCH"))?
        .as_secs() as i64;

    if now > shift_end {
        tracing::warn!(device_id = %payload.device_id, now, shift_end, "Verification failed: Shift has ended");
        return Err(on_shift_ended(&state.db, payload.device_id).await);
    }

    tracing::warn!("Step 4: Shift validity check passed");

    // 4. Verify the JWS compact signature
    verify_es256_jws(&payload.signed_nonce, &expected_nonce, &public_key_b64)?;

    // 5. Issue a new access token
    let user = crate::queries::user_queries::get_user_by_id(&state.db, user_id).await?;
    let jwt_svc = &state.jwt_svc;
    let access_token = jwt_svc
        .issue_access_token_with_shift(
            user_id,
            payload.device_id,
            user.role,
            shift_start.try_into().unwrap_or(0usize),
            shift_end.try_into().unwrap_or(0usize),
        )
        .map_err(AppError::Internal)?;

    tracing::info!(
        user_id = %user_id,
        device_id = %payload.device_id,
        "Token refresh verified — new access token issued"
    );

    Ok((
        axum::http::StatusCode::OK,
        Json(VerifyRefreshResponse { access_token }),
    ))
}

/// Verifies an ES256 (ECDSA P-256) compact JWS against a Base64-encoded public key.
fn verify_es256_jws(
    jws_compact: &str,
    expected_nonce: &str,
    public_key_b64: &str,
) -> Result<(), AppError> {
    tracing::warn!("--- [BACKEND] verify_es256_jws: Starting cryptographic verification ---");
    use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
    use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

    // Split JWS into 3 parts: header.payload.signature
    let parts: Vec<&str> = jws_compact.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(AppError::Unauthorized("Malformed JWS".into()));
    }

    let (header_b64, payload_b64, sig_b64) = (parts[0], parts[1], parts[2]);

    // Verify the payload matches the expected nonce
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| AppError::Unauthorized("Invalid JWS payload encoding".into()))?;
    let payload_str = std::str::from_utf8(&payload_bytes)
        .map_err(|_| AppError::Unauthorized("Invalid JWS payload".into()))?;

    if payload_str != expected_nonce {
        tracing::warn!(actual = %payload_str, expected = %expected_nonce, "Cryptographic failure: Nonce mismatch in JWS payload");
        return Err(AppError::Unauthorized("Nonce mismatch".into()));
    }

    tracing::warn!("JWS Payload matches expected nonce");

    // Decode the public key from Base64 -> JWK JSON -> VerifyingKey
    let pub_key_json = STANDARD
        .decode(public_key_b64)
        .map_err(|_| AppError::Unauthorized("Invalid public key encoding".into()))?;
    let pub_key_str = std::str::from_utf8(&pub_key_json)
        .map_err(|_| AppError::Unauthorized("Invalid public key data".into()))?;

    let public_key = p256::PublicKey::from_jwk_str(pub_key_str)
        .map_err(|_| AppError::Unauthorized("Invalid EC public key".into()))?;
    let verifying_key = VerifyingKey::from(&public_key);

    // Decode the signature (raw r||s, 64 bytes for P-256)
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| AppError::Unauthorized("Invalid JWS signature encoding".into()))?;

    let signature = Signature::from_slice(&sig_bytes).map_err(|_| {
        tracing::warn!("Cryptographic failure: Invalid ECDSA signature format");
        AppError::Unauthorized("Invalid ECDSA signature format".into())
    })?;

    // The message that was signed is "<header>.<payload>" (the JWS signing input)
    let signing_input = format!("{header_b64}.{payload_b64}");

    verifying_key.verify(signing_input.as_bytes(), &signature).map_err(|e| {
        tracing::warn!(error = %e, "Cryptographic failure: ES256 Signature verification failed");
        AppError::Unauthorized("Signature verification failed".into())
    })?;

    tracing::warn!("ES256 Signature successfully verified against public key");

    Ok(())
}

/// Change password for admin/org_admin users (used for forced password change on first login)
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = ChangePasswordResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse),
    ),
    tag = "auth",
    operation_id = "changePassword",
    security(("bearer_auth" = []))
)]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(requester): Extension<AuthenticatedAdmin>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.new_password != payload.confirm_password {
        return Err(AppError::bad_request("Passwords do not match"));
    }

    if payload.new_password.trim().is_empty() {
        return Err(AppError::bad_request("New password cannot be empty"));
    }

    // Fetch user's current state to determine if current password is required
    let user: (String, bool) = sqlx::query_as(
        "SELECT password_hash, must_change_password FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(requester.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let (user_password_hash, must_change_password) = user;

    // Security: Current password is REQUIRED unless this is a forced first-login password change
    if !must_change_password {
        // Normal password change - current password is MANDATORY
        let current_password = payload.current_password.as_ref().ok_or_else(|| {
            AppError::bad_request("Current password is required for password changes")
        })?;

        // Verify current password
        let is_valid =
            crate::utils::password::verify_password(current_password, &user_password_hash)
                .await
                .map_err(|_| AppError::bad_request("Invalid current password"))?;

        if !is_valid {
            return Err(AppError::bad_request("Invalid current password"));
        }
    }
    // If must_change_password is true (first login), current password is optional

    let new_hash = crate::utils::password::hash_password(&payload.new_password).await?;

    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1, must_change_password = FALSE, status = 'ACTIVE'::user_status
        WHERE id = $2
        "#,
    )
    .bind(&new_hash)
    .bind(requester.user_id)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    tracing::info!(user_id = %requester.user_id, "Password changed successfully");

    Ok((
        StatusCode::OK,
        Json(ChangePasswordResponse {
            message: "Password changed successfully".to_string(),
        }),
    ))
}
