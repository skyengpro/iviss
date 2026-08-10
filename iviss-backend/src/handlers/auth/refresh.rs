use super::*;

// ─────────────────────────────────────────────────────────────
//  Token Refresh — Challenge-Response with Device Signature
// ─────────────────────────────────────────────────────────────

use crate::app_cache::NONCE_TTL_SECS;

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
    if !auth::validate_agent_refresh_token(&state.db, &token_hash, device_id).await? {
        return Err(AppError::unauthorized_with_code(
            ErrorCode::SessionRevoked,
            "Invalid or expired refresh token",
        ));
    }

    // Generate a random 32-byte nonce
    let nonce = {
        let mut raw = [0u8; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut raw);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
    };

    // Store nonce in Moka cache with device_id as key, TTL handled automatically
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
    let row = auth::get_admin_refresh_context(&state.db, &token_hash).await?;

    let user_id = row.user_id;
    let role = row.role;
    let status = row.status;

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

    // Peek the nonce without consuming it yet: invalidating only happens once
    // the refresh token and signature are both verified below, so a failed
    // attempt (bad token, unknown device_id probed by a third party) doesn't
    // burn a legitimate challenge that the real device could still complete.
    let expected_nonce = state
        .app_cache
        .refresh_nonce
        .get(&payload.device_id)
        .await
        .ok_or_else(|| {
            tracing::warn!(device_id = %payload.device_id, "Verification failed: Nonce not found or expired");
            AppError::unauthorized_with_code(
                ErrorCode::NonceRetry,
                "Nonce expired or not found — request a new challenge",
            )
        })?;

    tracing::warn!(nonce = %expected_nonce, "Step 1: Nonce retrieved from Moka cache");

    // 2. Validate the refresh token
    let token_hash = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(payload.refresh_token.as_bytes());
        format!("{digest:x}")
    };

    let user_id =
        auth::get_refresh_token_user_id(&state.db, &token_hash, payload.device_id).await?;
    tracing::warn!(user_id = %user_id, "Step 2: Refresh token validated in database");

    // 3. Fetch the device's public key & shift metadata
    let device_row =
        auth::get_active_device_key_metadata(&state.db, payload.device_id, user_id).await?;

    tracing::warn!("Step 3: Device public key and shift metadata fetched");

    let public_key_b64 = device_row.public_key;
    let metadata = device_row.metadata;

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

    // Nonce fully verified — atomically consume it. `remove()` returns the
    // previous value if present, or `None` if another concurrent
    // verify_refresh (or an intervening request_refresh + verify_refresh)
    // has already consumed it. Returning `None` here means the same
    // signed_nonce is being replayed concurrently — reject the second
    // caller with a retriable classification so the client can re-challenge.
    let consumed = state
        .app_cache
        .refresh_nonce
        .remove(&payload.device_id)
        .await;
    if consumed.is_none() {
        tracing::warn!(
            device_id = %payload.device_id,
            "Nonce already consumed — concurrent verify_refresh or replay attempt"
        );
        return Err(AppError::unauthorized_with_code(
            ErrorCode::NonceRetry,
            "Nonce already consumed — request a new challenge",
        ));
    }

    // 5. Issue a new access token
    let user = crate::queries::users::get_user_by_id(&state.db, user_id).await?;
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
        return Err(AppError::unauthorized_with_code(
            ErrorCode::NonceRetry,
            "Nonce mismatch",
        ));
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
