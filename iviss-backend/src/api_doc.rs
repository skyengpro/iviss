use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::dto::{
    common::*,
    create_control::*,
    list_control::*,
    organizations::{Organization, OrganizationType},
    pending_submission::*,
    search_vehicle::*,
    stats::DashboardStats,
    users::{ProvisionUserRequest, UpdateUserRequest, UserProfile, UserRole, UserStatus},
};
use crate::errors::{AppErrorResponse, ErrorCode};

// ── Security scheme injector

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

// ── Central OpenAPI document ──
#[derive(OpenApi)]
#[openapi(
    info(
        title = "IVISS Backend API",
        version = "1.0.0",
        description = "Vehicle identification, carte grise submissions and back-office dashboard.",
    ),
    tags(
        (name = "vehicles", description = "Vehicle lookup and image upload"),
        (name = "controls", description = "Roadside control tracking"),
        (name = "stats", description = "Dashboard statistics"),
        (name = "users", description = "User profile management"),
        (name = "auth", description = "Authentication and registration"),
    ),
    paths(
        crate::handlers::search_vehicle::search_vehicle,
        crate::handlers::list_control::get_list_control,
        crate::handlers::list_control::create_control,
        crate::handlers::pending_submission::submit_vehicle,
        crate::handlers::pending_submission::list_pending_submissions,
        crate::handlers::pending_submission::get_pending_submission,
        crate::handlers::stats::get_dashboard_stats,
        crate::handlers::users::get_user_profile,
        crate::handlers::auth::login,
        crate::handlers::auth::register,
        crate::handlers::auth::logout,
        crate::handlers::user_management::provision_user,
        crate::handlers::user_management::list_users,
        crate::handlers::user_management::get_user,
        crate::handlers::user_management::update_user,
        crate::handlers::user_management::delete_user,
        crate::handlers::user_management::list_organizations,
    ),

    components(
        schemas(
            // ── common ──
            Status,
            IdentificationMode,
            // ── vehicle ──
            VehicleSearchRequest,
            VehicleSearchResult,
            VehicleInfo,
            OwnerInfo,
            StatusResults,
            InsuranceStatus,
            PoliceStatus,
            CustomsStatus,
            TechnicalStatus,
            SubmissionLocation,
            UploadResponse,
            // ── control ──
            ListControlResponse,
            ControlLocation,
            ControlResults,
            ControlAction,
            ActionType,
            CreateControlRequest,
            CreateControlResponse,
            CreatePendingSubmissionRequest,
            PendingSubmissionRequest,
            PendingSubmissionListItem,
            SubmissionStatus,
            DataEntryResponse,
            // ── errors ──
            AppErrorResponse,
            ErrorCode,
            DashboardStats,
            UserProfile,
            UserRole,
            // ── auth ──
            crate::handlers::auth::LoginRequest,
            crate::handlers::auth::AuthResponse,
            crate::handlers::auth::RegisterRequest,
            ProvisionUserRequest,
            UpdateUserRequest,
            UserStatus,
            Organization,
            OrganizationType,
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;
