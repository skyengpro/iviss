use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::dto::{
    auth::*,
    common::*,
    create_control::*,
    list_control::*,
    location::{UpdateLocationRequest, UpdateLocationResponse},
    organizations::{Organization, OrganizationType},
    pending_submission::*,
    scan::*,
    search_vehicle::*,
    stats::{
        ActivityData, ActivityFeedItemDto, ActivityFeedResponse, AgentLocationDto,
        ControlActivityPoint, ControlActivityResponse, DashboardRange, DashboardStats,
        RecentAlertItemDto, RecentAlertsResponse, TopAgentDto, TopAgentsResponse,
    },
    users::{
        DeviceStatus, ProvisionUserRequest, ResendActivationRequest, ResendActivationResponse,
        RestartSessionRequest, RestartSessionResponse, TerminateSessionRequest,
        TerminateSessionResponse, UpdateUserRequest, UserProfile, UserRole, UserStatus,
    },
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
        (name = "health", description = "Service health check"),
        (name = "vehicles", description = "Vehicle lookup and image upload"),
        (name = "controls", description = "Roadside control tracking"),
        (name = "scanning", description = "License plate OCR scanning"),
        (name = "photo", description = "Photo-based plate recognition"),
        (name = "stats", description = "Dashboard statistics"),
        (name = "users", description = "User profile management"),
        (name = "auth", description = "Authentication and registration"),
        (name = "admin", description = "Admin operations"),
    ),
    paths(
        crate::handlers::health::health_check,
        crate::handlers::search_vehicle::search_vehicle,
        crate::handlers::search_vehicle::search_vehicle_v1,
        crate::handlers::list_control::get_list_control,
        crate::handlers::list_control::get_list_control_paged,
        crate::handlers::list_control::create_control,
        crate::handlers::pending_submission::submit_vehicle,
        crate::handlers::pending_submission::submit_vehicle_v1,
        crate::handlers::pending_submission::list_pending_submissions,
        crate::handlers::pending_submission::get_pending_submission,
        crate::handlers::pending_submission::review_submission,
        crate::handlers::pending_submission::get_submission_audit_log,
        crate::handlers::scan::scan_plate,
        crate::handlers::photo::photo_plate,
        crate::handlers::stats::get_dashboard_stats,
        crate::handlers::stats::get_control_activity,
        crate::handlers::stats::get_top_agents,
        crate::handlers::stats::get_activity_feed,
        crate::handlers::stats::get_recent_alerts,
        crate::handlers::users::get_user_profile,
        crate::handlers::users::update_location,
        crate::handlers::auth::login,
        crate::handlers::auth::register,
        crate::handlers::auth::logout,
        crate::handlers::auth::request_daily_login,
        crate::handlers::auth::verify_daily_login,
        crate::handlers::device_management::suspend_device,
        crate::handlers::device_management::unsuspend_device,
        crate::handlers::auth::activate,
        crate::handlers::user_management::resend_activation_code,
        crate::handlers::auth::request_refresh,
        crate::handlers::auth::verify_refresh,
        crate::handlers::user_management::provision_user,
        crate::handlers::user_management::list_users,
        crate::handlers::user_management::get_user,
        crate::handlers::user_management::update_user,
        crate::handlers::user_management::delete_user,
        crate::handlers::user_management::list_organizations,
        crate::handlers::user_management::terminate_session,
        crate::handlers::user_management::restart_session,
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
            PagedControlsResponse,
            ControlLocation,
            ControlResults,
            ControlAction,
            ActionType,
            CreateControlRequest,
            CreateControlResponse,
            CreatePendingSubmissionRequest,
            PendingSubmissionDetail,
            PendingSubmissionListItem,
            SubmissionStatus,
            DataEntryResponse,
            ReviewSubmissionRequest,
            ReviewSubmissionResponse,
            VehicleDataEntry,
            SubmissionAuditLogEntry,
            SubmissionListQuery,
            // ── errors ──
            AppErrorResponse,
            ErrorCode,
            DashboardStats,
            ActivityData,
            DashboardRange,
            ControlActivityPoint,
            ControlActivityResponse,
            TopAgentDto,
            TopAgentsResponse,
            ActivityFeedItemDto,
            ActivityFeedResponse,
            RecentAlertItemDto,
            RecentAlertsResponse,
            AgentLocationDto,
            UserProfile,
            UserRole,
            UpdateLocationRequest,
            UpdateLocationResponse,
            // ── scanning ──
            ScanPlateResponse,
            ScanResultData,
            ScanErrorData,
            ImageUploadRequest,
            // ── auth ──
            LoginRequest,
            AuthResponse,
            RegisterRequest,
            ResendActivationRequest,
            ResendActivationResponse,
            RequestDailyLoginRequest,
            RequestDailyLoginResponse,
            VerifyDailyLoginRequest,
            VerifyDailyLoginResponse,
            // ── user management ──
            ActivateRequest,
            ActivateResponse,
            crate::dto::auth::RefreshRequest,
            crate::handlers::auth::RefreshChallengeResponse,
            crate::handlers::auth::VerifyRefreshRequest,
            crate::handlers::auth::VerifyRefreshResponse,
            ProvisionUserRequest,
            UpdateUserRequest,
            TerminateSessionRequest,
            TerminateSessionResponse,
            RestartSessionRequest,
            RestartSessionResponse,
            UserStatus,
            Organization,
            OrganizationType,
            crate::handlers::device_management::DeviceActionResponse,
            DeviceStatus,
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;
