// ── Shared DTO modules (always compiled) ──
pub mod common;
pub mod search_vehicle;

// ── API-only DTO modules ──
#[cfg(feature = "api")]
pub mod audit;
#[cfg(feature = "api")]
pub mod auth;
#[cfg(feature = "api")]
pub mod controls;
#[cfg(feature = "api")]
pub mod location;
#[cfg(feature = "api")]
pub mod organizations;
#[cfg(feature = "api")]
pub mod pending_submission;
#[cfg(feature = "api")]
pub mod scan;
#[cfg(feature = "api")]
pub mod stats;
#[cfg(feature = "api")]
pub mod users;
