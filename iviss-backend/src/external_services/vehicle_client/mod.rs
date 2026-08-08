//! Shared vehicle client module — used by both the API server and
//! the `s3-cache-sync` binary.
//!
//! Public API surface:
//! - [`types`]  — credential and response structs
//! - [`client`] — [`VehicleApiService`] with `new()` / `query_plate()`
//! - [`parser`] — HTML parsing helpers (pub for testing)

pub mod client;
pub mod parser;
pub mod types;

// Flatten the most-used items so callers can do:
//   use crate::external_services::vehicle_client::{VehicleApiService, VehicleApiError, …};
pub use client::VehicleApiService;
pub use types::{
    ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials, VehicleApiError, VehicleApiResponse,
    ExternalVehicle,
};
