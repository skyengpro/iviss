# Reorganize Backend for Two-Binary Architecture with Cargo Features

Restructure the `iviss-backend` crate so that it produces two independent binaries — the existing API server (`iviss-backend`) and a new S3 cache sync service (`s3-cache-sync`) — while sharing code through the library crate and keeping the sync binary lightweight via Cargo features.

## Key Design Decisions (Confirmed)

1. **Status computation stays API-only.** `build_status_results_from_api`, `calculate_overall_status`, `build_customs_status_from_api` move from `vehicle_client_service.rs` to `services/vehicle_status_service.rs` — the natural home for all status logic. The sync service stores only raw vehicle data (no status computation). When real insurance/technical/police APIs are added later, `vehicle_status_service.rs` will be the single place to wire them in.

2. **No dedup cache.** The sync service is the sole writer to S3 (1x/day via tokio cron scheduler). The API server only reads from S3 — never writes. The `moka` dedup cache in `S3VehicleDataCache` is removed.

3. **S3 cache stores raw vehicle info only.** The cached JSON contains the vehicle's identity data (plate, brand, model, chassis, owner, customs status from external API) — not computed `StatusResults`. The API computes status on-the-fly when serving from cache.

4. **Batch fetch strategy is deferred.** The sync binary is scaffolded with config + client initialization but the actual plate enumeration logic (`todo!()`) awaits a final decision.

---

## Proposed Changes

The work is split into **4 steps**, each leaving the codebase compilable and tests passing before moving to the next.

---

### Step 1 — Extract `vehicle_client` shared module

Promote `services/vehicle_client_service.rs` from a flat file into a reusable top-level module `src/vehicle_client/` so both binaries can import it. Status-related functions move to `vehicle_status_service.rs` instead.

#### [NEW] [vehicle_client/mod.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/vehicle_client/mod.rs)
Re-exports the public API from submodules.

#### [NEW] [vehicle_client/types.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/vehicle_client/types.rs)
Moved from `vehicle_client_service.rs`:
- `VehicleApiCredentials`, `ApiUserAuth`, `ExternalApiHeaderParms`
- `VehicleApiError`, `VehicleApiResponse`

#### [NEW] [vehicle_client/client.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/vehicle_client/client.rs)
Moved from `vehicle_client_service.rs`:
- `VehicleApiService` struct + `new()` + `query_plate()` + `parse_html_response()`

#### [NEW] [vehicle_client/parser.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/vehicle_client/parser.rs)
Moved from `vehicle_client_service.rs`:
- `html_to_text()`, `parse_label_value_lines()`, `split_brand_and_model()`, `parse_inline_customs_status()`, `clean_value()`, `decode_basic_html_entities()`, `is_vehicle_not_found_response()`
- All associated unit tests from `vehicle_client_service.rs`

#### [MODIFY] [services/vehicle_status_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/vehicle_status_service.rs)
Move these functions **from** `vehicle_client_service.rs` **into** this file:
- `build_status_results_from_api(vehicle_info: &VehicleInfo) -> StatusResults`
- `build_customs_status_from_api(customs_status: Option<&str>) -> CustomsStatus`
- The private `calculate_overall_status` already exists here — the duplicate in `vehicle_client_service.rs` is removed

These become methods on the existing `VehicleService` struct (or free functions in the module).

#### [DELETE] [vehicle_client_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/vehicle_client_service.rs)
Removed after migration.

#### [MODIFY] [lib.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/lib.rs)
Add `pub mod vehicle_client;` (top-level, alongside `s3_cache_layer`).

#### [MODIFY] [services/mod.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/mod.rs)
- Remove `pub mod vehicle_client_service;`
- Uncomment `pub mod vehicle_status_service;` (currently commented out)

#### [MODIFY] [config.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/config.rs)
Update the `pub use` re-export:
```diff
-pub use crate::services::vehicle_client_service::{
+pub use crate::vehicle_client::{
     ApiUserAuth, ExternalApiHeaderParms, VehicleApiCredentials,
 };
```

#### [MODIFY] [app_state.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/app_state.rs)
```diff
-use crate::services::vehicle_client_service::VehicleApiService;
+use crate::vehicle_client::VehicleApiService;
```

#### [MODIFY] [handlers/search_vehicle.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/handlers/search_vehicle.rs)
```diff
-    services::vehicle_client_service::{VehicleApiError, VehicleApiResponse, VehicleApiService},
+    vehicle_client::{VehicleApiError, VehicleApiResponse, VehicleApiService},
```
And update the `build_search_result` function to call status computation from `vehicle_status_service`:
```diff
-    let status_results = VehicleApiService::build_status_results_from_api(&vehicle_info);
+    let status_results = VehicleService::build_status_results_from_api(&vehicle_info);
```

**Verification**: `cargo check`, `cargo test` — all existing tests pass with zero logic changes.

---

### Step 2 — Populate `s3_cache_layer` shared module + simplify `vehicle_data_cache`

Move the S3 read/write, encryption, and partitioning logic out of `services/vehicle_data_cache.rs` into the existing `src/s3_cache_layer/` directory. Remove the dedup cache since the API no longer writes to S3.

#### [MODIFY] [s3_cache_layer/mod.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/s3_cache_layer/mod.rs)
Declares submodules and re-exports the public API.

#### [NEW] [s3_cache_layer/config.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/s3_cache_layer/config.rs)
Moved from `vehicle_data_cache.rs`:
- `S3CacheConfig` struct
- S3 client factory function: `pub async fn build_s3_client(config: &S3CacheConfig) -> Result<(aws_sdk_s3::Client, String)>` — extracts the shared client-building logic

#### [NEW] [s3_cache_layer/crypto.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/s3_cache_layer/crypto.rs)
Moved from `vehicle_data_cache.rs`:
- `encrypt(key, plaintext) -> Result<Vec<u8>>`
- `decrypt(key, data) -> Result<Vec<u8>>`
- All `payload_crypto` unit tests (round-trip, wrong key, short payload)

#### [NEW] [s3_cache_layer/types.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/s3_cache_layer/types.rs)
Moved from `vehicle_data_cache.rs`:
- `CachedEntry` (serde struct — **now stores only raw vehicle data, no `StatusResults`**)
- `CachedVehicleData` (public result struct returned on read)
- `cache_partition_for_plate()` + `REGION_CODES`, `S3_CACHE_PREFIX`, `OTHER_CACHE_PARTITION`
- `object_key(plate) -> Result<String>` (extracted from method to free function)
- Partition unit tests

#### [MODIFY] [s3_cache_layer/s3_reader.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/s3_cache_layer/s3_reader.rs)
Read path extracted from `S3VehicleDataCache::get_vehicle_data()`:
- `pub async fn read_vehicle_data(client, bucket, encryption_key, plate) -> Result<Option<CachedVehicleData>>`
- get_object → decrypt (if key set) → deserialize → return `CachedVehicleData`

#### [MODIFY] [s3_cache_layer/s3_writer.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/s3_cache_layer/s3_writer.rs)
Write path extracted from `S3VehicleDataCache::store_vehicle_data()`:
- `pub async fn write_vehicle_data(client, bucket, kms_key_id, encryption_key, plate, data) -> Result<()>`
- serialize → encrypt (if key set) → put_object (with optional SSE-KMS)
- **No dedup cache** — the sync service manages write frequency via its scheduler

#### [MODIFY] [services/vehicle_data_cache.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/vehicle_data_cache.rs)
Becomes a **thin read-only wrapper**:
- `VehicleDataCache` trait simplified: **only `get_vehicle_data()`** — `store_vehicle_data()` is removed
- `S3VehicleDataCache` struct simplified: holds S3 client + bucket + encryption_key — **no dedup cache**
- Delegates to `s3_cache_layer::s3_reader::read_vehicle_data()`
- All crypto, types, partitioning, write logic removed (lives in `s3_cache_layer` now)

#### [MODIFY] [handlers/search_vehicle.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/handlers/search_vehicle.rs)
- Remove `cache_vehicle_search_result()` function (API no longer writes to S3)
- Remove the `cache_vehicle_search_result(&state, plate.clone(), response.clone())` call from the success path
- When serving from S3 cache fallback, compute status on-the-fly using `VehicleService::build_status_results_from_api()`

#### [MODIFY] [app_cache.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/app_cache.rs)
Remove the `vehicle_dedup` cache field if it was only used by `S3VehicleDataCache`.

#### [MODIFY] [main.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/main.rs)
Update `S3VehicleDataCache::from_config()` call to no longer pass the dedup cache.

#### [MODIFY] [config.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/config.rs)
```diff
-pub use crate::services::vehicle_data_cache::S3CacheConfig;
+pub use crate::s3_cache_layer::S3CacheConfig;
```

#### [MODIFY] [tests/vehicle_data_cache_tests.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/tests/vehicle_data_cache_tests.rs)
- Update imports to use `crate::s3_cache_layer::S3CacheConfig`
- Remove the `second_store_is_deduped` test (dedup cache removed)
- Update remaining tests: call writer then reader separately, since the trait no longer has `store_vehicle_data`

**Verification**: `cargo check`, `cargo test` — all existing tests pass, including the MinIO integration tests.

---

### Step 3 — Cargo features to isolate dependencies

Gate API-only dependencies behind a Cargo feature so that `s3-cache-sync` compiles without OCR, web framework, OpenAPI, or database code.

#### [MODIFY] [Cargo.toml](file:///home/lonsti-ws/Documents/iviss/iviss-backend/Cargo.toml)
```toml
[features]
default = ["api"]
api = [
    "dep:axum",
    "dep:sqlx",
    "dep:leptess",
    "dep:image",
    "dep:imageproc",
    "dep:utoipa",
    "dep:utoipa-swagger-ui",
    "dep:tower-http",
    "dep:moka",
    "dep:metrics",
    "dep:metrics-exporter-prometheus",
    "dep:opentelemetry",
    "dep:opentelemetry_sdk",
    "dep:opentelemetry-otlp",
    "dep:tracing-opentelemetry",
    "dep:lettre",
    "dep:jsonwebtoken",
    "dep:rsa",
    "dep:p256",
    "dep:ecdsa",
    "dep:argon2",
    "dep:hmac",
    "dep:sha2",
]
```

Each dependency listed above gets `optional = true` in `[dependencies]`.

**Shared dependencies** (always compiled, used by both binaries):
`tokio`, `reqwest`, `aws-config`, `aws-sdk-s3`, `aes-gcm`, `serde`, `serde_json`, `dotenvy`, `config`, `tracing`, `tracing-subscriber`, `anyhow`, `thiserror`, `once_cell`, `regex`, `time`, `uuid`, `rand`, `base64`, `urlencoding`, `async-trait`

#### [MODIFY] [lib.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/lib.rs)
Gate API-only modules:
```rust
// ── Shared modules (always compiled) ──
pub mod vehicle_client;
pub mod s3_cache_layer;
pub mod dto;
pub mod utils;

// ── API-only modules (gated behind "api" feature) ──
#[cfg(feature = "api")] pub mod api_doc;
#[cfg(feature = "api")] pub mod app_cache;
#[cfg(feature = "api")] pub mod app_state;
#[cfg(feature = "api")] pub mod config;
#[cfg(feature = "api")] pub mod db;
#[cfg(feature = "api")] pub mod errors;
#[cfg(feature = "api")] pub mod feature_flags;
#[cfg(feature = "api")] pub mod handlers;
#[cfg(feature = "api")] pub mod middleware;
#[cfg(feature = "api")] pub mod models;
#[cfg(feature = "api")] pub mod queries;
#[cfg(feature = "api")] pub mod routes;
#[cfg(feature = "api")] pub mod services;
#[cfg(feature = "api")] pub mod telemetry;
#[cfg(feature = "api")] #[cfg(test)] pub mod tests;
```

> [!WARNING]
> Shared modules (`vehicle_client`, `s3_cache_layer`, `dto`, `utils`) must NOT import from gated modules. Known issue: `config.rs` currently imports from `services::vehicle_client_service` and `services::vehicle_data_cache`. After Steps 1-2, those imports point to the shared modules instead, but `config.rs` also loads SMS/email/JWT/database config that the sync binary doesn't need.
>
> **Solution**: Gate `config.rs` under `#[cfg(feature = "api")]`. The sync binary reads its own env vars directly in `s3-cache-sync.rs` (it only needs S3 config + vehicle API credentials — both types live in shared modules).

#### [MODIFY] [bin/s3-cache-sync.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/bin/s3-cache-sync.rs)
Scaffold the binary with real initialization but placeholder batch logic:
```rust
//! S3 Cache Sync Service
//!
//! Periodically fetches vehicle data from the external API and populates
//! the S3 cache layer. Designed to run as a long-lived service with a
//! tokio-cron-scheduler (1x/day).
//!
//! Build: cargo build --bin s3-cache-sync --no-default-features

use iviss_backend::s3_cache_layer;
use iviss_backend::vehicle_client::VehicleApiService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Init tracing (minimal, stdout only — no OpenTelemetry)
    // 2. Load S3CacheConfig + VehicleApiCredentials from env vars
    // 3. Build VehicleApiService (shared module)
    // 4. Build S3 client via s3_cache_layer::build_s3_client()
    // 5. TODO: Set up tokio-cron-scheduler for 1x/day execution
    // 6. TODO: Batch-fetch logic (awaiting final decision)
    //    - enumerate plates by prefix (CE, NW, LT, ...)
    //    - for each plate: query_plate → s3_writer::write_vehicle_data
    todo!("Batch fetch strategy pending decision")
}
```

> [!NOTE]
> `tokio-cron-scheduler` will need to be added as a shared dependency (or sync-only if we add a `sync` feature). This is deferred until the batch logic is designed.

**Verification**:
- `cargo check` (default features — API binary) ✅
- `cargo check --bin s3-cache-sync --no-default-features` ✅
- `cargo test` (default features — all existing tests pass) ✅

---

### Step 4 — Dockerfile for s3-cache-sync + docker-compose service

Create a lean production Dockerfile for the sync service (no OCR libraries).

#### [NEW] [Dockerfile.s3-cache-sync](file:///home/lonsti-ws/Documents/iviss/iviss-backend/Dockerfile.s3-cache-sync)
Key differences from the API Dockerfile:
- **No OCR packages** (`libtesseract-dev`, `libleptonica-dev`, `tesseract-ocr-eng`) → much smaller image
- Builds with `--no-default-features` → no axum/sqlx/utoipa linked
- No `EXPOSE` — not a server
- No `HEALTHCHECK` — long-running scheduler that doesn't expose HTTP

#### [MODIFY] [docker-compose.yml](file:///home/lonsti-ws/Documents/iviss/docker-compose.yml)
Add a dev-profile service:
```yaml
  s3-cache-sync:
    profiles: ["dev"]
    build:
      context: ./iviss-backend
      dockerfile: Dockerfile.s3-cache-sync
    container_name: iviss-s3-cache-sync
    restart: unless-stopped
    environment:
      # Only S3 + vehicle API env vars needed
      EXTERNAL_API_BASE_URL: ${EXTERNAL_API_BASE_URL}
      EXTERNAL_API_USERNAME: ${EXTERNAL_API_USERNAME}
      # ... (subset of backend-environment)
      S3_CACHE_BUCKET: ${S3_CACHE_BUCKET:-iviss-vehicle-cache}
      S3_CACHE_REGION: ${S3_CACHE_REGION:-eu-west-1}
      S3_CACHE_ENDPOINT_URL: ${S3_CACHE_ENDPOINT_URL:-}
      S3_CACHE_ENCRYPTION_KEY: ${S3_CACHE_ENCRYPTION_KEY:-}
      AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID:-}
      AWS_SECRET_ACCESS_KEY: ${AWS_SECRET_ACCESS_KEY:-}
    depends_on:
      minio:
        condition: service_healthy
    logging: *default-logging
    networks:
      - iviss-network
```

---

## Verification Plan

### Automated Tests
After each step:
```bash
# Step 1-2: full compilation + all tests
cargo check
cargo test

# Step 3: verify both binaries compile
cargo check                                              # API (default features)
cargo check --bin s3-cache-sync --no-default-features    # sync (no API deps)
cargo test                                               # all tests still pass

# Step 4: Docker builds
docker build -f Dockerfile -t iviss-backend:test .
docker build -f Dockerfile.s3-cache-sync -t s3-cache-sync:test .
```

### Manual Verification
- Confirm `docker images` shows the sync image is significantly smaller than the API image (no OCR libs)
- Run `docker compose --profile dev up backend` and verify the API still works normally
