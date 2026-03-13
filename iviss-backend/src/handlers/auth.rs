use crate::app_state::AppState;
use crate::dto::users::{UserProfile, UserRole};
use crate::errors::AppError;
use crate::services::activation_service::ActivationService;
use crate::services::jwt_service::JwtService;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use base64::Engine;
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendActivationRequest {
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendActivationResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivateRequest {
    pub badge_id: String,
    pub activation_code: String,
    pub device_id: Uuid,
    pub public_key_base64: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivateResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
    pub device_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
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
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Development/test: Allow seed admin credentials to log in via the web back office.
    // Agents use the OTP activation flow instead.
    if (payload.email == "admin" || payload.email == "admin01") && payload.password == "admin123" {
        // LOOKUP the actual user in the DB instead of hardcoding UUIDs
        let user_row = sqlx::query(
            r#"
            SELECT u.id, u.username, u.email, u.full_name as name, u.role::TEXT, u.organization_id, 
                   o.name as organization_name, u.badge_id, u.phone_number, u.status::TEXT
            FROM users u
            LEFT JOIN organizations o ON u.organization_id = o.id
            WHERE u.email = 'admin01@iviss.gov' OR u.email = 'admin@iviss.gov'
            AND u.deleted_at IS NULL
            "#,
        )
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::unauthorized("Admin user not found in database"))?;

        let admin_id: Uuid = user_row.get("id");
        let org_id: Uuid = user_row.get("organization_id");
        let username: String = user_row.get("username");
        let email: Option<String> = user_row.get("email");
        let name: String = user_row.get("name");
        let _role_str: String = user_row.get("role");
        let org_name: Option<String> = user_row.get("organization_name");
        let badge_id: Option<String> = user_row.get("badge_id");
        let phone_number: Option<String> = user_row.get("phone_number");
        let status_str: String = user_row.get("status");

        let jwt_svc = JwtService::new(&state.jwt_private_key_pem).map_err(AppError::Internal)?;
        let device_id = Uuid::new_v4(); // Virtual device – one per web session

        // Compute shift window (same TTL as agent activation)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::internal_error("System time before UNIX_EPOCH"))?
            .as_secs();
        let shift_start: i64 = now.try_into().unwrap_or(0);
        let shift_end: i64 = now
            .saturating_add(SHIFT_TTL.as_secs())
            .try_into()
            .unwrap_or(0);

        let token = jwt_svc
            .issue_access_token_with_shift(
                admin_id,
                device_id,
                UserRole::Admin,
                shift_start.try_into().unwrap_or(0),
                shift_end.try_into().unwrap_or(0),
            )
            .map_err(AppError::Internal)?;

        // Register the virtual device so the auth middleware's device_is_active check passes.
        sqlx::query(
            r#"
            INSERT INTO devices (id, user_id, public_key, status, metadata)
            VALUES (
                $1, $2, $3, 'ACTIVE'::device_status,
                jsonb_build_object('shift_start', $4, 'shift_end', $5)
            )
            ON CONFLICT (id) DO UPDATE SET
                status   = 'ACTIVE'::device_status,
                metadata = EXCLUDED.metadata
            "#,
        )
        .bind(device_id)
        .bind(admin_id)
        .bind(device_id.to_string())
        .bind(shift_start)
        .bind(shift_end)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

        return Ok((
            StatusCode::OK,
            Json(AuthResponse {
                token,
                user: UserProfile {
                    id: admin_id,
                    username,
                    email,
                    name: name.clone(),
                    role: UserRole::Admin,
                    organization_id: org_id,
                    organization: org_name,
                    badge_id,
                    phone_number,
                    avatar_initials: Some(name.chars().next().unwrap_or('A').to_string()),
                    status: status_str
                        .parse()
                        .unwrap_or(crate::dto::users::UserStatus::Active),
                    is_active: true,
                },
            }),
        ));
    }

    Err(AppError::unauthorized("Invalid credentials"))
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

/// Send activation code via SMS to a pending agent
#[utoipa::path(
    post,
    path = "/auth/send-activation",
    request_body = SendActivationRequest,
    responses(
        (status = 201, description = "Activation code sent", body = SendActivationResponse),
        (status = 404, description = "User not found", body = AppErrorResponse),
        (status = 400, description = "Bad request", body = AppErrorResponse)
    ),
    tag = "auth",
    operation_id = "sendActivationCode"
)]
pub async fn send_activation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendActivationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch agent from DB
    let user = sqlx::query(
        r#"
        SELECT id,
               phone_number,
               role::TEXT AS role,
               status::TEXT AS status
        FROM users
        WHERE id = $1
        AND deleted_at IS NULL
        "#,
    )
    .bind(payload.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let user_id: Uuid = user.get("id");
    let phone_number: String = user.get("phone_number");
    let role: String = user.get("role");
    let status: String = user.get("status");

    // Only agents can receive an activation code
    if role != "agent" {
        return Err(AppError::BadRequest(
            "Activation is only available for agents".into(),
        ));
    }

    // Agent must be in PENDING_ACTIVATION, SUSPENDED, or ACTIVE status
    if status != "PENDING_ACTIVATION" && status != "SUSPENDED" && status != "ACTIVE" {
        return Err(AppError::BadRequest(format!(
            "User is not in an activatable state — current status: {}",
            status
        )));
    }

    // Build ActivationService from shared state resources
    let activation_svc = ActivationService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );

    // Generate, store and send the activation code via SMS
    activation_svc
        .generate_and_send(&user_id, &phone_number)
        .await
        .map_err(AppError::Internal)?;

    Ok((
        StatusCode::CREATED,
        Json(SendActivationResponse {
            message: "Activation code sent successfully".into(),
        }),
    ))
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
    if user_status != "PENDING_ACTIVATION" && user_status != "SUSPENDED" && user_status != "ACTIVE"
    {
        return Err(AppError::BadRequest(format!(
            "User is not in an activatable state — current status: {}",
            user_status
        )));
    }

    let activation_svc = ActivationService::new(
        state.redis.clone(),
        state.sms_pvd.clone(),
        state.pepper.clone(),
    );
    activation_svc
        .validate(&user_id, &payload.activation_code)
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
