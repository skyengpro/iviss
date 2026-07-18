//! Database setup for iviss-test-api.
//!
//! Responsibilities:
//! - Create the `test_api` schema and `vehicles` table (idempotent DDL).
//! - Apply the seed SQL file embedded at compile time.
//! - Expose the `Vehicle` model and query helpers.

use anyhow::Context;
use sqlx::PgPool;

// ── Schema DDL ───────────────────────────────────────────────────────────────

const SCHEMA_DDL: &str = r#"
CREATE SCHEMA IF NOT EXISTS test_api;

CREATE TABLE IF NOT EXISTS test_api.vehicles (
    id             SERIAL PRIMARY KEY,
    plate_number   TEXT NOT NULL,
    chassis_number TEXT,
    mark_and_type  TEXT,
    engine_power   TEXT,
    owner_name     TEXT,
    nps_status     TEXT,
    customs_status TEXT
);

-- Case-insensitive, space-stripped unique index for plate lookup.
CREATE UNIQUE INDEX IF NOT EXISTS vehicles_plate_upper_idx
    ON test_api.vehicles (UPPER(REPLACE(plate_number, ' ', '')));
"#;

// Seed SQL embedded at compile time so the binary is self-contained.
const SEED_SQL: &str = include_str!("seeds/vehicles.sql");

// ── Public model ─────────────────────────────────────────────────────────────

/// A vehicle record from the test database.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Vehicle {
    pub plate_number:   String,
    pub chassis_number: Option<String>,
    pub mark_and_type:  Option<String>,
    pub engine_power:   Option<String>,
    pub owner_name:     Option<String>,
    pub nps_status:     Option<String>,
    pub customs_status: Option<String>,
}

// ── Init ─────────────────────────────────────────────────────────────────────

/// Apply the schema DDL and seed data. Safe to call on every startup.
pub async fn init(pool: &PgPool) -> anyhow::Result<()> {
    // Apply schema (idempotent)
    sqlx::raw_sql(SCHEMA_DDL)
        .execute(pool)
        .await
        .context("Failed to apply test_api schema DDL")?;

    tracing::info!("test_api schema ready");

    // Apply seed (INSERT ... ON CONFLICT DO NOTHING)
    sqlx::raw_sql(SEED_SQL)
        .execute(pool)
        .await
        .context("Failed to apply vehicle seed data")?;

    tracing::info!("Vehicle seed data applied");
    Ok(())
}

// ── Queries ──────────────────────────────────────────────────────────────────

/// Look up a single vehicle by plate number.
///
/// Lookup is case-insensitive and ignores spaces, so `ce568lr`, `CE568LR`
/// and `CE 568 LR` all resolve to the same row.
pub async fn find_by_plate(pool: &PgPool, plate: &str) -> anyhow::Result<Option<Vehicle>> {
    let normalised = plate.replace(' ', "").to_uppercase();

    sqlx::query_as::<_, Vehicle>(
        r#"
        SELECT plate_number, chassis_number, mark_and_type,
               engine_power, owner_name, nps_status, customs_status
        FROM   test_api.vehicles
        WHERE  UPPER(REPLACE(plate_number, ' ', '')) = $1
        "#,
    )
    .bind(&normalised)
    .fetch_optional(pool)
    .await
    .context("DB error during plate lookup")
}

/// Return all vehicles whose normalised plate starts with the given prefix.
///
/// The prefix is uppercased before matching so `ce` == `CE`.
pub async fn find_by_prefix(pool: &PgPool, prefix: &str) -> anyhow::Result<Vec<Vehicle>> {
    let normalised_prefix = format!("{}%", prefix.replace(' ', "").to_uppercase());

    sqlx::query_as::<_, Vehicle>(
        r#"
        SELECT plate_number, chassis_number, mark_and_type,
               engine_power, owner_name, nps_status, customs_status
        FROM   test_api.vehicles
        WHERE  UPPER(REPLACE(plate_number, ' ', '')) LIKE $1
        ORDER  BY plate_number
        "#,
    )
    .bind(&normalised_prefix)
    .fetch_all(pool)
    .await
    .context("DB error during prefix batch query")
}
