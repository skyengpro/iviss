use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::dto::{
    common::{IdentificationMode, Status},
    record_control::{
        ActionType, ControlAction, ControlLocation, ControlRecord, ControlResults,
        CreateControlRequest, GpsPosition,
    },
    vehicle::{
        CustomsStatus, InsuranceStatus, OwnerInfo, PendingVehicleSubmission, PoliceStatus,
        StatusResults, SubmissionLocation, TechnicalStatus, UploadResponse, VehicleInfo,
        VehicleSearchRequest, VehicleSearchResult,
    },
};
use crate::errors::AppErrorResponse;

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

// ── Central OpenAPI document ──────────────────────────────────────────────────
// paths() is intentionally empty here.
// Each handler will be added incrementally in Steps 4–7
// using the #[utoipa::path(...)] macro on the handler function,
// then registered here.

#[derive(OpenApi)]
#[openapi(
    info(
        title = "IVISS Backend API",
        version = "1.0.0",
        description = "Vehicle identification, carte grise submissions and back-office dashboard.",
    ),
    tags(
        (name = "auth",     description = "Authentication — login and token refresh"),
        (name = "vehicles", description = "Vehicle lookup and gray-card image upload"),
        (name = "controls", description = "Roadside control tracking"),
        (name = "stats",    description = "Back-office dashboard statistics"),
        (name = "users",    description = "Authenticated user profile"),
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
            PendingVehicleSubmission,
            SubmissionLocation,
            UploadResponse,
            // ── control ──
            CreateControlRequest,
            GpsPosition,
            ControlRecord,
            ControlLocation,
            ControlResults,
            ControlAction,
            ActionType,
            // ── errors ──
            AppErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;
