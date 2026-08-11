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
use crate::queries::auth;
use rand::RngCore;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use uuid::Uuid;

pub mod activate;
pub mod change_password;
pub mod daily_login;
pub mod login;
pub mod logout;
pub mod refresh;
pub mod router;

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
pub use refresh::{on_shift_ended, request_refresh, verify_refresh};
