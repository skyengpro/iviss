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

// refresh_token handler removed - replaced by request_refresh

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

// RefreshResponse removed - no longer used

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

    // Agent must be in PENDING_ACTIVATION or SUSPENDED status
    if status != "PENDING_ACTIVATION" && status != "SUSPENDED" {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation or suspended — current status: {}",
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
    if user_status != "PENDING_ACTIVATION" && user_status != "SUSPENDED" {
        return Err(AppError::BadRequest(format!(
            "User is not pending activation or suspended — current status: {}",
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

// ─────────────────────────────────────────────────────────────
//  Token Refresh — Challenge-Response with Device Signature
// ─────────────────────────────────────────────────────────────

const NONCE_TTL_SECS: u64 = 60;
const NONCE_KEY_PREFIX: &str = "refresh_nonce";

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
/// Validates the refresh token, generates a nonce, stores it in Redis,
/// and returns it to the client for signing.
#[utoipa::path(
    post,
    path = "/auth/refresh",
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
    // Hash the incoming refresh token to match against DB
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(payload.refresh_token.as_bytes());
        format!("{:x}", digest)
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
    .bind(payload.device_id)
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

    // Store nonce in Redis with device_id as key, TTL 60s
    let nonce_key = format!("{}:{}", NONCE_KEY_PREFIX, payload.device_id);
    {
        use deadpool_redis::redis::AsyncCommands;
        let mut conn = state
            .redis
            .get()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection error: {}", e)))?;
        conn.set_ex::<_, _, ()>(&nonce_key, &nonce, NONCE_TTL_SECS)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis SET error: {}", e)))?;
    }

    tracing::info!(
        device_id = %payload.device_id,
        "Refresh nonce issued (TTL={}s)",
        NONCE_TTL_SECS
    );

    Ok((
        StatusCode::OK,
        Json(RefreshChallengeResponse { nonce }),
    ))
}

/// Step 2 of the challenge-response refresh flow.
///
/// Verifies the signed nonce against the device's registered public key,
/// then issues a new access token.
#[utoipa::path(
    post,
    path = "/auth/refresh/verify",
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

    // 1. Retrieve and consume the nonce from Redis (one-time use)
    let nonce_key = format!("{}:{}", NONCE_KEY_PREFIX, payload.device_id);
    let stored_nonce: Option<String> = {
        use deadpool_redis::redis::AsyncCommands;
        let mut conn = state
            .redis
            .get()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection error: {}", e)))?;
        let val: Option<String> = conn
            .get(&nonce_key)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis GET error: {}", e)))?;
        // Delete immediately to prevent replay
        conn.del::<_, ()>(&nonce_key).await.ok();
        val
    };

    let expected_nonce = stored_nonce.ok_or_else(|| {
        tracing::warn!(device_id = %payload.device_id, "Verification failed: Nonce not found or expired");
        AppError::Unauthorized("Nonce expired or not found — request a new challenge".into())
    })?;

    tracing::warn!(nonce = %expected_nonce, "Step 1: Nonce retrieved and consumed from Redis");

    // 2. Validate the refresh token
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(payload.refresh_token.as_bytes());
        format!("{:x}", digest)
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
    let jwt_svc =
        JwtService::new(&state.jwt_private_key_pem).map_err(AppError::Internal)?;
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
        StatusCode::OK,
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
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    verifying_key.verify(signing_input.as_bytes(), &signature).map_err(|e| {
        tracing::warn!(error = %e, "Cryptographic failure: ES256 Signature verification failed");
        AppError::Unauthorized("Signature verification failed".into())
    })?;

    tracing::warn!("ES256 Signature successfully verified against public key");

    Ok(())
}
