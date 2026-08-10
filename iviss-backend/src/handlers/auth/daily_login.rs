use super::*;

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

    if user.status == "PENDING_ACTIVATION" {
        return Err(AppError::unauthorized("Account pending activation"));
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

    let (shift_start_minutes, shift_end_minutes) =
        crate::queries::organization_queries::get_organization_work_time_cached(
            &state.db,
            &state.app_cache,
            org_id,
        )
        .await?;

    let local_offset = time::UtcOffset::from_hms(1, 0, 0).unwrap_or(time::UtcOffset::UTC);
    let now_local = time::OffsetDateTime::now_utc().to_offset(local_offset);
    let current_minute_of_day = (now_local.hour() as u32) * 60 + (now_local.minute() as u32);

    // Shift window: stored as minutes since midnight (inclusive start, exclusive end)
    let fmt_time = |mins: u32| -> String { format!("{:02}:{:02}", mins / 60, mins % 60) };
    if current_minute_of_day < shift_start_minutes || current_minute_of_day >= shift_end_minutes {
        return Err(AppError::unauthorized(format!(
            "Outside shift hours — login is available from {} to {} local time",
            fmt_time(shift_start_minutes),
            fmt_time(shift_end_minutes)
        )));
    }

    let device_opt =
        auth_queries::get_device_by_user_optional(&state.db, payload.device_id, user.id).await?;

    let device = match device_opt {
        Some(d) => d,
        None => {
            let device_exists =
                auth_queries::check_device_exists(&state.db, payload.device_id).await?;
            if device_exists {
                return Err(AppError::bad_request(
                    " Incompatible Badge ID for this device. Please check your Badge ID.",
                ));
            } else {
                return Err(AppError::NotFound(
                    "Device is not registered. Please re-activate.".into(),
                ));
            }
        }
    };

    if device.status == "SUSPENDED" {
        return Err(AppError::unauthorized(
            "Device suspended — contact your administrator",
        ));
    }

    // Check for administrative termination cooldown
    if let (Some(revoked_at), dev_status) = (device.revoked_at, device.status) {
        if dev_status != "PENDING" {
            // Assume UTC for the stored TIMESTAMP (project convention)
            let local_offset = time::UtcOffset::from_hms(1, 0, 0).unwrap_or(time::UtcOffset::UTC);
            let revoked_local = revoked_at.to_offset(local_offset);
            let now = OffsetDateTime::now_utc().to_offset(local_offset);

            if revoked_local.date() == now.date() {
                return Err(AppError::Forbidden(
                format!("Session terminated by administrator. Please wait until your next shift (tomorrow at {}) to request a new code.", fmt_time(shift_start_minutes))
            ));
            }
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
        (status = 401, description = "Invalid OTP, expired, or device suspended or pending", body = AppErrorResponse),
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
    // but if it IS registered, it must not be suspended
    if device_status == "SUSPENDED" {
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
