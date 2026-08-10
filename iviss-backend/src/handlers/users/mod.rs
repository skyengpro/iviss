use crate::app_state::AppState;
use crate::dto::location::{UpdateLocationRequest, UpdateLocationResponse};
use crate::dto::users::{
    ProvisionUserRequest, ProvisionUserResponse, ResendActivationRequest, ResendActivationResponse,
    ResendOrgAdminPasswordRequest, ResendOrgAdminPasswordResponse, RestartSessionRequest,
    RestartSessionResponse, TerminateSessionRequest, TerminateSessionResponse, UpdateUserRequest,
    UserRole, UserStatus,
};
use crate::errors::AppError;
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::rbac::AuthenticatedAdmin;
use crate::queries::organizations::list_organizations as list_organizations_query;
use crate::queries::users::{
    create_org_admin_user_with_temp_password, get_activation_resend_user,
    get_org_admin_password_resend_user, get_user_by_id, hard_delete_user,
    list_users as list_users_query, list_users_by_org, mark_user_pending_and_revoke_refresh_tokens,
    update_org_admin_temporary_password, update_user as update_user_query,
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::Engine;
use std::sync::Arc;
use uuid::Uuid;

pub mod activation;
pub mod location;
pub mod profile;
pub mod provisioning;
pub mod sessions;

pub use activation::{__path_resend_activation_code, __path_resend_org_admin_password};
pub use activation::{resend_activation_code, resend_org_admin_password};
pub use location::__path_update_location;
pub use location::update_location;
pub use profile::__path_get_user_profile;
pub use profile::get_user_profile;
pub use provisioning::{
    __path_delete_user, __path_get_user, __path_list_org_users, __path_list_organizations,
    __path_list_users, __path_provision_org_user, __path_provision_user, __path_update_user,
};
pub use provisioning::{
    delete_user, get_user, list_org_users, list_organizations, list_users, provision_org_user,
    provision_user, update_user,
};
pub use sessions::{__path_restart_session, __path_terminate_session};
pub use sessions::{restart_session, terminate_session};
