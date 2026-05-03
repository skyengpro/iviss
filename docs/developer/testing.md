# IVISS Testing Guide

This document explains how to run IVISS tests and quality checks locally, and how those checks map to CI.

## 1) Test Stack Overview

IVISS has separate backend and frontend test stacks.

Backend:

- Rust test runner: `cargo test`  (or use nextest for fast tests runner)
- Async tests: Tokio
- HTTP/app integration tests: Axum + Tower
- Database integration tests: Testcontainers + PostgreSQL
- External provider tests: local mocks and `mockito`
- Coverage in CI: `cargo-llvm-cov` + `nextest`
- Static checks: `cargo fmt`, `cargo clippy`, `cargo audit`, `cargo doc`

Frontend:

- Unit/component test runner: Vitest
- DOM environment: `jsdom`
- React tests: Testing Library
- Coverage provider: V8
- Static checks: ESLint, Prettier, TypeScript, Vite build
- Generated API client checks: OpenAPI codegen before build/lint/type/test jobs

CI workflows live in `.github/workflows/`.

## 2) Quick Local Checks

Use these commands before opening a PR.

Backend:

```bash
cd iviss-backend
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

Frontend:

```bash
cd frontend
npm run lint:check
npm run prettier:check
npm run ts:check
npm run test
npm run build
```

If a frontend change depends on API types, regenerate the client first:

```bash
cd frontend
npm run codegen
```

## 3) Backend Tests

Backend tests are located in:

```text
iviss-backend/src/tests/
```

Some query-level tests also live next to implementation modules under `iviss-backend/src/queries/`.

Run all backend tests:

```bash
cd iviss-backend
cargo test
```

Run a specific test module or test name:

```bash
cd iviss-backend
cargo test auth_queries_tests
cargo test test_admin_login_success
```

Run tests with output:

```bash
cd iviss-backend
cargo test -- --nocapture
```

Database-backed tests start isolated PostgreSQL containers with Testcontainers and then run `sqlx::migrate!("./migrations")`. Docker must be running for these tests.

Use mock providers for SMS/email in tests. Do not call real SMS, email, OCR, or external government providers from automated tests.

## 4) Backend Quality and Coverage

Run backend formatting:

```bash
cd iviss-backend
cargo fmt --all -- --check
```

Run Clippy:

```bash
cd iviss-backend
cargo clippy -- -D warnings
```

Run Rust security audit:

```bash
cd iviss-backend
cargo audit
```

The CI audit currently ignores specific known advisories in `.github/workflows/backend-ci.yml`. If an advisory must be ignored, document why in the workflow or Cargo audit config instead of hiding the failure locally.

Build Rust docs:

```bash
cd iviss-backend
cargo doc --no-deps --verbose
```

Run the backend local CI helper:

```bash
cd iviss-backend
./scripts/test_flow.sh
```

This script builds, tests with coverage instrumentation, checks formatting, runs Clippy, runs audit, builds docs, and generates an HTML coverage report.

Generate backend coverage manually:

```bash
cd iviss-backend
./scripts/coverage.sh
```

CI uses `cargo-llvm-cov` with `nextest` and uploads an HTML coverage artifact.

## 5) Frontend Tests

Frontend tests are colocated with code under `__tests__/` folders or `src/test/`.

Run all frontend tests:

```bash
cd frontend
npm run test
```

Run tests in watch mode:

```bash
cd frontend
npm run test:watch
```

Run coverage:

```bash
cd frontend
npm run coverage
```

Vitest configuration lives in `frontend/vitest.config.ts`.

Important frontend test helpers:

- `frontend/src/test/setup.ts`: global `jest-dom` setup and browser API shims.
- `frontend/src/test/queryWrapper.tsx`: fresh React Query wrapper for hook/component tests.

Frontend test conventions:

- Use Testing Library queries that match user-visible behavior.
- Use `MemoryRouter` for route-dependent components.
- Use `createQueryWrapper()` for hooks/components that depend on TanStack Query.
- Mock generated OpenAPI hooks/services at the boundary for component tests.
- Stub browser APIs such as `fetch`, `navigator.geolocation`, `matchMedia`, storage, camera, and IndexedDB behavior when needed.
- Clear `localStorage`, mocks, timers, and global stubs between tests.

## 6) Frontend Quality Checks

Run ESLint:

```bash
cd frontend
npm run lint:check
```

Run Prettier check:

```bash
cd frontend
npm run prettier:check
```

Run TypeScript:

```bash
cd frontend
npm run ts:check
```

Run production build:

```bash
cd frontend
npm run build
```

Run OpenAPI codegen:

```bash
cd frontend
npm run codegen
```

The frontend CI runs OpenAPI codegen before build, lint, Prettier, TypeScript, and unit test jobs. Generated files under `frontend/src/openapi-rq/` should be regenerated, not edited manually.

## 7) CI Checks

Backend CI runs on pull requests and pushes to `main` and `dev` when backend files or the backend workflow change.

Backend jobs:

- Gitleaks secret scan.
- Rust build.
- Test and coverage with `cargo-llvm-cov` and `nextest`.
- Rust doc tests.
- HTML coverage report generation.
- Rust formatting.
- Clippy with warnings denied.
- Cargo audit.
- Rust documentation build.

Frontend CI runs on pull requests and pushes to `main` and `dev` when frontend files or the frontend workflow change.

Frontend jobs:

- `npm ci --legacy-peer-deps`
- OpenAPI codegen.
- Vite build.
- ESLint.
- Prettier check.
- TypeScript check.
- Unit tests with coverage.
- SonarQube scan using frontend LCOV coverage.

Docker and deploy workflows are separate from unit test workflows, but test failures should be fixed before relying on deployment jobs.

## 8) Secret and Security Testing

Gitleaks is configured through `.gitleaks.toml` and runs in backend CI.

Run a local secret scan from the repository root when touching env, deployment, or docs with example secrets:

```bash
docker run --rm -v "$PWD:/path" zricethezav/gitleaks:latest detect --source="/path" -v --redact --config=/path/.gitleaks.toml
```

Rules:

- Never commit real JWT private keys, API keys, database passwords, provider credentials, or production URLs.
- Prefer safe placeholders in docs and `.env.example` files.
- If a test needs cryptographic material, use clearly fake test fixtures and ensure they are allowlisted intentionally.

## 9) What to Test by Change Type

Backend endpoint change:

- Handler success and error cases.
- DTO serialization/deserialization.
- Auth/RBAC behavior if protected.
- Query behavior if database-backed.
- OpenAPI/codegen impact if frontend consumes it.

Database change:

- Migration applies cleanly to an empty database.
- Query tests cover new columns, constraints, filters, and indexes.
- Seed data still works when `SEED_DATA=true`.
- Existing workflow tests still pass.

Frontend UI change:

- Component rendering and user-visible states.
- Hook behavior and loading/error/success states.
- Route guard behavior if navigation or roles changed.
- API hook integration with generated types.

Auth/session change:

- Access token and refresh behavior.
- Session revocation and logout behavior.
- Device activation and daily login flows.
- Role-specific access paths.

Infrastructure or deployment change:

- Relevant script syntax and dry-run checks where available.
- Docker build if container behavior changes.
- Documentation updates for any operator-facing command or secret.

## 10) Testing Checklist

Before opening a PR:

- Run the quick local checks for the changed subsystem.
- Add or update tests for new behavior and regressions.
- Regenerate OpenAPI client when the API contract changes.
- Review test data for secrets or production identifiers.
- Update developer docs if setup, test commands, CI behavior, or generated artifacts change.
