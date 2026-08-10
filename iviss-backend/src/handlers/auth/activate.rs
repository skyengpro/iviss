use super::*;

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
#[instrument(name = "auth.activate", skip(state, payload), fields(badge_id = %payload.badge_id))]
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

    let user_row = auth::get_activation_user_by_badge(&mut tx, &payload.badge_id).await?;

    let user_id = user_row.id;
    let user_role = user_row.role;
    let user_org_id = user_row.organization_id;
    let user_status = user_row.status;

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

    let org_id = user_org_id
        .ok_or_else(|| AppError::forbidden("Agent must belong to an organization to activate"))?;

    let (shift_start_minutes, shift_end_minutes) =
        crate::queries::organizations::get_organization_work_time_cached(
            &state.db,
            &state.app_cache,
            org_id,
        )
        .await?;

    let localt_time_offset = time::UtcOffset::from_hms(1, 0, 0)
        .map_err(|_| AppError::internal_error("Failed to build UTC+1 offset"))?;

    let today_local = time::OffsetDateTime::now_utc()
        .to_offset(localt_time_offset)
        .date();

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

    auth::mark_user_active(&mut tx, user_id).await?;

    auth::upsert_active_device(
        &mut tx,
        payload.device_id,
        user_id,
        &payload.public_key_base64,
        shift_start,
        shift_end,
    )
    .await?;

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

    auth::insert_refresh_token(
        &mut tx,
        &refresh_token_hash,
        user_id,
        payload.device_id,
        refresh_expires_at,
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

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
