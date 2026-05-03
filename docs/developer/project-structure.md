# IVISS Project Structure

This document explains the repository layout and where to implement changes depending on feature scope.

## 1) Repository Overview

```text
iviss/
├── .github/            # CI/CD workflows (frontend, backend, release, docker, deploy)
├── docs/               # Product, architecture, operational, and developer docs
├── frontend/           # React + TypeScript application
├── iviss-backend/      # Rust + Axum API service
├── infra/              # Terraform + Ansible deployment code
├── monitoring/         # Prometheus + Grafana configuration
├── scripts/            # Shared utility scripts (e.g., OpenAPI fetch)
├── docker-compose.yml  # Local dev/prod-like compose orchestration
└── README.md           # Root project entry documentation
```

## 2) Backend Structure (`iviss-backend/`)

Main folders:

- `src/main.rs`: backend startup, config loading, migrations, route assembly, Swagger setup.
- `src/routes.rs`: route registration and middleware composition.
- `src/handlers/`: HTTP handlers (request/response layer).
- `src/services/`: business services and external/provider integrations (SMS, email, JWT, OCR, vehicle).
- `src/queries/`: SQL query layer and data access operations.
- `src/dto/`: request/response and transfer shapes.
- `src/models/`: domain models and DB-facing structures.
- `src/middleware/`: auth/RBAC/CORS middleware.
- `src/db/`: pool initialization and seed logic.
- `src/tests/`: integration and module tests.
- `src/bin/`: utility binaries (`seed`, `export_openapi`, `openapi_gen`).
- `migrations/`: SQL migrations.
- `seeds/`: SQL seed files to populate tests data in the database.
- `.sqlx/`: SQLx offline metadata to allow compiler time check to the database schema.

Backend entry points:

- Runtime server: `src/main.rs`
- Library modules: `src/lib.rs`
- Route composition: `src/routes.rs`

## 3) Frontend Structure (`frontend/`)

Main folders:

- `src/main.tsx`: React app mount, router bootstrap, error boundary.
- `src/App.tsx`: root providers (React Query, auth, app initializer, toasters).
- `src/router/`: route definitions and auth guards.
- `src/pages/`: page-level screens (`auth`, `mobile`, `backoffice`).
- `src/components/`: reusable UI and feature components.
- `src/hooks/`: custom hooks (`api`, `auth`, `feature`, `ui`).
- `src/services/`: API client adapters, auth/device helpers, metrics, mocks.
- `src/openapi-rq/`: generated API client code and query helpers.
- `src/i18n/`: localization configuration and dictionaries.
- `src/utils/` and `src/lib/`: shared helpers/utilities.
- `src/test/` and `**/__tests__/`: frontend tests.

Frontend config and build files:

- `vite.config.ts`
- `tailwind.config.ts`
- `eslint.config.js`
- `vitest.config.ts`
- `openapi.json` (OpenAPI input used by frontend codegen)

## 4) Infrastructure and Operations

`infra/`:

- `terraform/`: infrastructure provisioning (Lightsail and related resources).
- `ansible/`: server configuration and deployment playbooks/roles.
- `scripts/`: deploy/destroy/bootstrap helper scripts.

`monitoring/`:

- `prometheus/prometheus.yml`: scrape and monitoring config.
- `grafana/`: provisioning and dashboard definitions.

## 5) Documentation Layout

`docs/` contains:

- System/architecture docs
- Deployment/operations docs
- Domain/process docs
- Developer docs under `docs/developer/`

Use `docs/developer/` for implementation-facing documentation.

## 6) Generated and Derived Artifacts

Treat these as generated/derived unless the change explicitly targets generation logic:

- `frontend/src/openapi-rq/` (generated from OpenAPI)
- `frontend/openapi.json` (API contract snapshot for frontend codegen)
- `iviss-backend/.sqlx/` (SQLx metadata for offline checks)
- Build outputs such as `frontend/dist/`, `frontend/dev-dist/`, `iviss-backend/target/`

If backend API changes:

1. Update backend handlers/DTOs/OpenAPI annotations.
2. Regenerate/update OpenAPI spec.
3. Regenerate frontend client (`openapi-rq`) and adapt consuming code.

## 7) Typical Change Paths

New backend endpoint:

1. Add/adjust DTO (`src/dto/`).
2. Implement handler (`src/handlers/`).
3. Add/adjust query/service (`src/queries/` or `src/services/`).
4. Register route in `src/routes.rs`.
5. Add tests in `src/tests/`.

Frontend feature consuming API:

1. Use generated client from `src/openapi-rq/`.
2. Add service/hook logic in `src/services/` or `src/hooks/api/`.
3. Render in `src/pages/` and `src/components/`.
4. Add/adjust tests in `__tests__/`.

Deployment/infrastructure change:

1. Update Terraform in `infra/terraform/` if infrastructure topology changes.
2. Update Ansible in `infra/ansible/` for host/runtime config changes.
3. Validate relevant GitHub workflow under `.github/workflows/`.

## 8) Ownership Guidance (Practical)

- Prefer local, targeted edits in the module where behavior originates.
- Keep route/auth/security changes synchronized across `routes`, middleware, and tests.
- Keep docs updated in the same PR when commands, env vars, APIs, or workflows change.
