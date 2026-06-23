# IVISS Database Development Guide

This document explains how IVISS developers work with the PostgreSQL database, SQLx migrations, seed data, and query code.

## 1) Database Overview

IVISS uses PostgreSQL as the application database.

Main database sources:

- `iviss-backend/migrations/`: ordered SQLx migrations.
- `iviss-backend/src/db/`: pool initialization, migration-time helpers, bootstrap seed, and dev seed runner.
- `iviss-backend/src/queries/`: SQL data access functions used by handlers and services.
- `iviss-backend/seeds/seed_data.sql`: optional local/demo data.

At backend startup, `iviss-backend/src/main.rs` performs this sequence:

1. Load and validate environment configuration.
2. Initialize the PostgreSQL pool from `DATABASE_URL`.
3. Run SQLx migrations from `iviss-backend/migrations/`.
4. Run the optional admin bootstrap seed.
5. Run optional development seed data when `SEED_DATA=true`.
6. Cache required database data in `AppCache`.

The Rust application currently initializes only the main `DATABASE_URL` connection. `EXTERNAL_DATABASE_URL` appears in compose/env configuration for external-registry integration, but it is not currently loaded by `Config` (waiting client external database credentials)

## 2) Local Database Runtime

The recommended local setup is Docker Compose from the repository root:

```bash
docker compose --profile dev up -d --build
```

Useful services:

- PostgreSQL container: `iviss-db`
- Backend container: `iviss-backend`
- Adminer: `http://localhost:8081`

The backend connects to PostgreSQL through the Docker network using the `DATABASE_URL` assembled in `docker-compose.yml`.

Minimum database-related environment values:

- `POSTGRES_USER`
- `POSTGRES_PASSWORD`
- `POSTGRES_DB`
- `DATABASE_URL` when running the backend outside Docker
- `SEED_DATA=true` only when local/demo seed data should be loaded
- `ADMIN_BOOTSTRAP_*` values only when the first admin should be created automatically

Do not commit real local passwords, production database URLs, or exported database dumps.

## 3) Core Tables and Domains

The current schema is created from migrations, not from a separate ORM model. Important domains include:

- Organizations: `organizations`
- Users and roles: `users`, `user_role`, `user_status`
- Devices and auth sessions: `devices`, `refresh_tokens`, `access_token_blacklist`
- Audit trail: `audit_logs`, `submission_audit_log`
- Vehicle registry: `vehicles`, `vehicle_owners`, `vehicle_statuses`
- Controls: `control_records`, `control_actions`
- Pending carte grise workflow: `pending_submissions`
- Agent location tracking: `agent_locations`

Use `docs/schema.md` for the schema narrative and ERD. Use the SQL migrations as the final source of truth when exact columns, constraints, indexes, or enum values matter.

## 4) Migrations

Migrations live in `iviss-backend/migrations/` and are applied in filename order. Existing filenames use timestamp prefixes:

```text
YYYYMMDDHHMMSS_description.sql
```

Create a new migration from `iviss-backend/`:

```bash
cd iviss-backend
sqlx migrate add describe_the_change
```

Run migrations manually when needed:

```bash
cd iviss-backend
sqlx migrate run
```

Revert the last migration in local development only:

```bash
cd iviss-backend
sqlx migrate revert
```

Migration rules:

- Prefer additive changes for shared branches and deployed environments.
- Never edit a migration that has already been pushed or applied by another developer; create a new follow-up migration.
- Include indexes for new query patterns, especially filters by `organization_id`, `user_id`, `created_at`, status fields, and lookup keys.
- Keep enum changes explicit with `ALTER TYPE ... ADD VALUE IF NOT EXISTS` when extending existing PostgreSQL enums.
- Use transactions where possible, but remember some PostgreSQL operations have transaction limitations depending on version and operation type.
- Backfill data carefully when adding non-null columns to populated tables.

## 5) Query Layer

Database access should stay in `iviss-backend/src/queries/` unless there is a strong reason to keep a small query local to a test or seed.

Current query modules include:

- `auth_queries.rs`: login, devices, refresh tokens, activation, session state.
- `user_queries.rs`: user management and account lifecycle.
- `organization_queries.rs`: organization CRUD and constraints.
- `vehicle_queries.rs`: vehicle and status lookup.
- `control_queries.rs`: control creation and history.
- `submission_queries.rs`: pending submission review workflow.
- `stats_queries.rs`: dashboard and reporting queries.
- `audit_queries.rs`: audit log listing/export.
- `location_queries.rs`: agent location updates and lookup.
- `session_queries.rs`: session/device termination helpers.

Query conventions:

- Bind parameters with SQLx instead of string interpolation.
- Return `Result<_, AppError>` at application query boundaries.
- Map SQLx failures through `AppError::database`.
- Use transactions for multi-step state transitions, such as approving or rejecting submissions.
- Use `FOR UPDATE` when a workflow must lock the current row state before changing it.
- Keep tenant scoping explicit in queries that serve organization-specific screens.

## 6) Seed Data

There are two seed mechanisms.

Admin bootstrap seed:

- Implemented in `iviss-backend/src/db/seed_admin.rs`.
- Runs at startup after migrations.
- Requires all `ADMIN_BOOTSTRAP_*` env vars.
- Skips if any required bootstrap env var is missing.
- Skips if an admin already exists.
- Hashes the configured password before storing it.
- Logs failures without crashing the app.

Development seed data:

- Implemented in `iviss-backend/src/db/seed_data.rs`.
- Loads `iviss-backend/seeds/seed_data.sql`.
- Runs only when `SEED_DATA=true`.
- Intended for local/demo workflows, not production.
- Uses fixed sample IDs and cleanup statements to stay repeatable.

If seed data changes table shape, update the seed SQL in the same PR as the migration.

## 7) Reset and Maintenance Scripts

Scripts live in `iviss-backend/scripts/`.

Useful scripts:

- `init_db.sh`: starts local database services and runs SQLx setup.

The truncate and drop scripts are destructive. Use them only for local development databases.

If using `init_db.sh`, ensure `DATABASE_URL` is set correctly or that the database is reachable on the script fallback host/port. The main Docker Compose backend path does not require direct host access to PostgreSQL.

## 8) SQLx Offline Metadata

The backend enables `SQLX_OFFLINE=true` in Docker Compose. SQLx offline metadata lives under:

```text
iviss-backend/.sqlx/
```

When adding compile-time checked SQLx macros in the future, refresh offline metadata before committing:

```bash
cd iviss-backend
cargo sqlx prepare --workspace
```

Most current query code uses dynamic `sqlx::query(...)` and `sqlx::query_as(...)`, so compile-time query metadata may not cover every SQL statement.

## 9) Testing Database Changes

Run backend checks from `iviss-backend/`:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Database-related tests use Testcontainers for isolated PostgreSQL instances in several modules. For schema changes, add or update tests around the query module or handler flow that owns the behavior.

For local manual verification:

```bash
docker compose --profile dev up -d --build
docker compose logs -f backend
```

Then inspect data through Adminer at:

```text
http://localhost:8081
```

## 10) Database Change Checklist

Before opening a PR that changes the database:

- A new migration exists for every schema change.
- Existing pushed migrations are not edited.
- Query code and DTOs match the new schema.
- Indexes support new list, search, or dashboard access patterns.
- Seed data still runs when `SEED_DATA=true`.
- Admin bootstrap behavior remains idempotent.
- Backend tests cover the changed query or workflow.
- `docs/schema.md` is updated for meaningful entity/relationship changes.
- Developer docs are updated when setup, migration, seed, or reset workflows change.
