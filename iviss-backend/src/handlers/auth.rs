use crate::app_state::AppState;
use crate::dto::auth::{
    ActivateRequest, ActivateResponse, AuthResponse, LoginRequest, RefreshRequest, RefreshResponse,
    RegisterRequest,
};
use crate::dto::auth::{
    RequestDailyLoginRequest, RequestDailyLoginResponse, VerifyDailyLoginRequest,
    VerifyDailyLoginResponse,
};
use base64::Engine;

use crate::dto::users::{UserProfile, UserRole};
use crate::errors::AppError;
use crate::queries::auth_queries;
use crate::services::jwt_service::JwtService;
use crate::services::otp_service::OtpService;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use rand::RngCore;
use sqlx::Row;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
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

/// Refresh access token using refresh token
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = RefreshResponse),
        (status = 401, description = "Invalid refresh token", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "refreshToken"
)]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.refresh_token.trim().is_empty() {
        return Err(AppError::bad_request("refreshToken is required"));
    }

    let refresh_token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(payload.refresh_token.as_bytes());
        format!("{:x}", digest)
    };

    let row = sqlx::query(
        r#"
        SELECT
            rt.user_id              AS user_id,
            d.metadata              AS metadata,
            d.status::text          AS device_status
        FROM refresh_tokens rt
        JOIN devices d ON d.id = rt.device_id
        WHERE rt.token_hash = $1
          AND rt.device_id = $2
          AND rt.revoked = FALSE
          AND rt.expires_at > NOW()
        "#,
    )
    .bind(&refresh_token_hash)
    .bind(payload.device_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::unauthorized("Invalid refresh token"))?;

    let user_id: Uuid = row.get("user_id");
    let device_status: String = row.get("device_status");
    if device_status != "ACTIVE" {
        return Err(AppError::unauthorized("Device is not active"));
    }

    let metadata: serde_json::Value = row.get("metadata");
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
        let _ =
            crate::queries::auth_queries::mark_device_inactive(&state.db, payload.device_id).await;
        return Err(AppError::unauthorized("Shift ended"));
    }

    let user = crate::queries::user_queries::get_user_by_id(&state.db, user_id).await?;
    let jwt_svc = JwtService::new(&state.jwt_private_key_pem).map_err(AppError::Internal)?;
    let access_token = jwt_svc
        .issue_access_token_with_shift(
            user_id,
            payload.device_id,
            user.role,
            shift_start.try_into().unwrap_or(0usize),
            shift_end.try_into().unwrap_or(0usize),
        )
        .map_err(AppError::Internal)?;

    Ok((StatusCode::OK, Json(RefreshResponse { access_token })))
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "loginUser"
)]
pub async fn login(Json(_payload): Json<LoginRequest>) -> Result<impl IntoResponse, AppError> {
    // MOCK LOGIN
    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token: "mock-jwt-token".to_string(),
            user: UserProfile {
                id: uuid::Uuid::new_v4(),
                username: "admin".to_string(),
                email: Some("admin@iviss.com".to_string()),
                name: "Admin User".to_string(),
                role: UserRole::Admin,
                organization_id: uuid::Uuid::new_v4(),
                organization: Some("IVISS HQ".to_string()),
                badge_id: Some("ADMIN-01".to_string()),
                phone_number: Some("+237 600 000 000".to_string()),
                avatar_initials: Some("AU".to_string()),
                status: crate::dto::users::UserStatus::Active,
                is_active: true,
            },
        }),
    ))
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created", body = AuthResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "registerUser"
)]
pub async fn register(Json(payload): Json<RegisterRequest>) -> Result<impl IntoResponse, AppError> {
    // MOCK REGISTER
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token: "mock-jwt-token".to_string(),
            user: UserProfile {
                id: uuid::Uuid::new_v4(),
                username: payload
                    .email
                    .split('@')
                    .next()
                    .unwrap_or("user")
                    .to_string(),
                email: Some(payload.email),
                name: payload.full_name,
                role: payload.role,
                organization_id: uuid::Uuid::new_v4(),
                organization: Some("Independent".to_string()),
                badge_id: Some("TEMP-01".to_string()),
                phone_number: None,
                avatar_initials: Some("NU".to_string()),
                status: crate::dto::users::UserStatus::Active,
                is_active: true,
            },
        }),
    ))
}

/// Logout and invalidate session
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = String)
    ),
    tag = "auth",
    operation_id = "logoutUser",
    security(("bearer_auth" = []))
)]
pub async fn logout() -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json("Logout successful".to_string())))
}

/// Activate an agent account by validating OTP and registering device public key
#[utoipa::path(
    post,
    path = "/auth/activate",
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
               role::TEXT AS role,
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
    let user_role: String = user_row.get("role");
    let user_status: String = user_row.get("status");

    if user_role != "agent" {
        return Err(AppError::BadRequest(
            "Activation is only available for agents".into(),
        ));
    }
    if user_status != "PENDING_ACTIVATION" && user_status != "SUSPENDED" {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation or suspended — current status: {}",
            user_status
        )));
    }

    let otp_svc = OtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );
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
        format!("{:x}", digest)
    };
    let refresh_expires_at = {
        let dt = OffsetDateTime::now_utc() + time::Duration::days(30);
        time::PrimitiveDateTime::new(dt.date(), dt.time())
    };

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

    let jwt_svc = JwtService::new(&state.jwt_private_key_pem).map_err(AppError::Internal)?;
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
    path = "/auth/request-daily-login",
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
    let user = auth_queries::get_user_by_badge_id(&state.db, &payload.badge_id).await?;

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

    let device = auth_queries::get_device_by_user(&state.db, payload.device_id, user.id).await?;

    if device.status == "SUSPENDED" {
        return Err(AppError::unauthorized(
            "Device suspended — contact your administrator",
        ));
    }

    let otp_svc = OtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    otp_svc.request_otp(&user.id, &user.phone_number).await?;

    tracing::info!(
        target: "daily_login",
        user_id = %user.id,
        device_id = %device.id,
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
    path = "/auth/verify-daily-login",
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
    if payload.otp.trim().is_empty() {
        return Err(AppError::bad_request("otp is required"));
    }

    let row = sqlx::query(
        r#"
        SELECT
            u.id              AS user_id,
            u.role::TEXT      AS user_role,
            u.status::TEXT    AS user_status,
            d.status::TEXT    AS device_status
        FROM users u
        JOIN devices d
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
    let user_role: String = row.get("user_role");
    let user_status: String = row.get("user_status");
    let device_status: String = row.get("device_status");

    // ── Status checks
    if user_role != "agent" {
        return Err(AppError::unauthorized(
            "Daily login is only available for agents",
        ));
    }
    if user_status == "SUSPENDED" {
        return Err(AppError::unauthorized(
            "Account suspended — contact your administrator",
        ));
    }
    if device_status == "SUSPENDED" {
        return Err(AppError::unauthorized(
            "Device suspended — contact your administrator",
        ));
    }

    // ── Validate OTP
    let otp_svc = OtpService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );
    otp_svc.validate_otp(&user_id, &payload.otp).await?;

    // ── Compute static shift bounds (UTC+1 local time)
    let localt_time_offset = time::UtcOffset::from_hms(1, 0, 0)
        .map_err(|_| AppError::internal_error("Failed to build UTC+1 offset"))?;

    let today_local = time::OffsetDateTime::now_utc()
        .to_offset(localt_time_offset)
        .date();

    let shift_start_time = time::Time::from_hms(state.shift_start_hour as u8, 0, 0)
        .map_err(|_| AppError::internal_error("Invalid shift_start_hour in config"))?;

    let shift_end_time = time::Time::from_hms(state.shift_end_hour as u8, 0, 0)
        .map_err(|_| AppError::internal_error("Invalid shift_end_hour in config"))?;

    let shift_start: i64 =
        time::OffsetDateTime::new_in_offset(today_local, shift_start_time, localt_time_offset)
            .unix_timestamp();

    let shift_end: i64 =
        time::OffsetDateTime::new_in_offset(today_local, shift_end_time, localt_time_offset)
            .unix_timestamp();

    // ── Issue access token (15 min, carries today's static shift bounds) ──────
    let jwt_svc = JwtService::new(&state.jwt_private_key_pem).map_err(AppError::Internal)?;

    let role = user_role
        .parse::<crate::dto::users::UserRole>()
        .map_err(|_| AppError::internal_error("Invalid user role in database"))?;

    let access_token = jwt_svc
        .issue_access_token_with_shift(
            user_id,
            payload.device_id,
            role,
            shift_start.try_into().unwrap_or(0usize),
            shift_end.try_into().unwrap_or(0usize),
        )
        .map_err(AppError::Internal)?;

    // ── Check if a valid refresh token already exists for this device
    let has_valid_refresh: bool =
        auth_queries::has_valid_refresh_token(&state.db, payload.device_id).await?;

    // ── Conditionally build new refresh token
    let new_refresh: Option<(String, String, time::PrimitiveDateTime)> = if !has_valid_refresh {
        let raw = {
            let mut bytes = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut bytes);
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
        };
        let hash = {
            use sha2::Digest;
            format!("{:x}", sha2::Sha256::digest(raw.as_bytes()))
        };
        let expires_at = {
            let dt = time::OffsetDateTime::now_utc() + time::Duration::days(30);
            time::PrimitiveDateTime::new(dt.date(), dt.time())
        };
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
