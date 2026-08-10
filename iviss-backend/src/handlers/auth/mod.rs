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
use tracing::instrument;

use crate::dto::users::{UserProfile, UserRole, UserStatus};
use crate::errors::{AppError, ErrorCode};
use crate::middleware::rbac::AuthenticatedAdmin;
use crate::queries::auth_queries;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod activate;
pub mod change_password;
pub mod daily_login;
pub mod login;
pub mod logout;
pub mod refresh;

pub use activate::__path_activate;
pub use activate::activate;
pub use change_password::__path_change_password;
pub use change_password::change_password;
pub use daily_login::{__path_request_daily_login, __path_verify_daily_login};
pub use daily_login::{request_daily_login, verify_daily_login};
pub use login::__path_login;
pub use login::login;
pub use logout::__path_logout;
pub use logout::logout;
pub use refresh::{__path_request_refresh, __path_verify_refresh};
pub use refresh::{
    request_refresh, verify_refresh, RefreshChallengeResponse, VerifyRefreshRequest,
    VerifyRefreshResponse,
};

/// Logic to execute when a shift has ended.
/// Marks the device as inactive and returns an unauthorized error.
pub async fn on_shift_ended(pool: &sqlx::PgPool, device_id: Uuid) -> AppError {
    tracing::warn!(%device_id, "shift: ended logic triggered");

    if let Err(err) = crate::queries::auth_queries::mark_device_inactive(pool, device_id).await {
        tracing::error!(%device_id, error = %err, "shift: failed to mark device inactive");
    }

    AppError::unauthorized_with_code(ErrorCode::ShiftEnded, "Shift ended")
}
