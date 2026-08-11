pub mod config;
pub mod crypto;
mod s3_queue;
pub mod s3_reader;
pub mod s3_writer;
pub mod types;

pub use config::{build_s3_client, S3CacheConfig};
pub use s3_queue::{enqueue_plate, list_queued_plates, mark_unregistered, remove_marker};
pub use types::CachedVehicleData;
