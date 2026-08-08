// ── Shared modules (always compiled) ──
pub mod dto;
pub mod external_services;
pub mod s3_cache_layer;
pub mod utils;

// ── API-only modules (gated behind "api" feature) ──
#[cfg(feature = "api")]
pub mod api_doc;
#[cfg(feature = "api")]
pub mod app_cache;
#[cfg(feature = "api")]
pub mod app_state;
#[cfg(feature = "api")]
pub mod config;
#[cfg(feature = "api")]
pub mod db;
#[cfg(feature = "api")]
pub mod errors;
#[cfg(feature = "api")]
pub mod handlers;
#[cfg(feature = "api")]
pub mod middleware;
#[cfg(feature = "api")]
pub mod models;
#[cfg(feature = "api")]
pub mod queries;
#[cfg(feature = "api")]
pub mod routes;
#[cfg(feature = "api")]
pub mod services;
#[cfg(feature = "api")]
pub mod telemetry;
#[cfg(feature = "api")]
#[cfg(test)]
pub mod tests;
