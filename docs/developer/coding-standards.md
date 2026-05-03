# IVISS Coding Standards

This document defines the practical coding standards for IVISS contributors. It complements automated checks; when in doubt, prefer the existing code style in the module being changed.

## 1) General Principles

- Keep changes focused and reviewable.
- Prefer existing project patterns over new abstractions.
- Keep behavior changes close to the module that owns the behavior.
- Update tests and documentation in the same PR when behavior, commands, schemas, APIs, or workflows change.
- Avoid committing generated churn unless the generated artifact is part of the contract, such as `frontend/openapi.json` or `frontend/src/openapi-rq/` after an API change.
- Do not commit real secrets, local credentials, database dumps, or production identifiers.

## 2) Repository Boundaries

Use the existing ownership boundaries:

- Backend HTTP layer: `iviss-backend/src/handlers/`
- Backend business/provider logic: `iviss-backend/src/services/`
- Backend database access: `iviss-backend/src/queries/`
- Backend DTOs/API shapes: `iviss-backend/src/dto/`
- Backend routes/middleware: `iviss-backend/src/routes.rs` and `iviss-backend/src/middleware/`
- Frontend pages: `frontend/src/pages/`
- Frontend reusable components: `frontend/src/components/`
- Frontend API hooks: `frontend/src/hooks/api/`
- Frontend generated API client: `frontend/src/openapi-rq/`
- Infrastructure code: `infra/`
- Monitoring assets: `monitoring/`
- Developer documentation: `docs/developer/`

Generated files should not be edited manually. Change the source and regenerate.

## 3) Backend Rust Standards

Run before opening backend PRs:

```bash
cd iviss-backend
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

Rust conventions:

- Use `rustfmt` formatting; there is no custom rustfmt config in the repo.
- Prefer explicit, typed errors through `AppError` at request/query boundaries.
- Use `anyhow::Result` for startup or infrastructure setup where rich context matters.
- Add context to fallible startup/config operations with `.context(...)`.
- Keep handlers focused on HTTP extraction, validation, auth context, and response mapping.
- Put reusable business logic in services.
- Put SQL data access in query modules.
- Use `tracing` for operational logs; do not use `println!` in runtime code.
- Avoid panics in request paths. Reserve `expect` for startup invariants that should fail fast.
- Keep comments rare and useful; explain non-obvious behavior, not line-by-line mechanics.

## 4) Backend API Standards

For API changes:

- Register routes in `iviss-backend/src/routes.rs`.
- Document handlers with `#[utoipa::path]`.
- Keep path, method, and security annotations aligned with `routes.rs`.
- Define request/response structures in `iviss-backend/src/dto/`.
- Add DTOs and handlers to `iviss-backend/src/api_doc.rs` when they are part of OpenAPI.
- Use `#[serde(rename_all = "camelCase")]` for JSON payloads unless an existing endpoint intentionally uses another format.
- Use shared error responses through `AppErrorResponse`.
- Regenerate frontend OpenAPI artifacts when the API contract changes.

Protected endpoints should declare:

```rust
security(("bearer_auth" = []))
```

## 5) Backend Database Standards

Database conventions:

- Add a new SQLx migration for every schema change.
- Do not edit pushed migrations.
- Bind SQL parameters with SQLx instead of interpolating values into SQL strings.
- Use transactions for multi-step state changes.
- Lock rows with `FOR UPDATE` when reviewing or transitioning workflow state.
- Keep organization/tenant scoping explicit in queries.
- Add indexes for new list, search, dashboard, or auth lookup patterns.
- Keep seed data aligned with schema changes.

Run database-impacting tests with Docker available because Testcontainers starts PostgreSQL containers.

## 6) Frontend TypeScript Standards

Run before opening frontend PRs:

```bash
cd frontend
npm run lint:check
npm run prettier:check
npm run ts:check
npm run test
npm run build
```

Formatting comes from `frontend/.prettierrc`:

- Semicolons: enabled.
- Quotes: single quotes.
- Indentation: 2 spaces.
- Trailing commas: ES5.
- Print width: 100.
- Arrow parentheses: always.

TypeScript conventions:

- Prefer `@/` imports for source aliases when crossing feature folders.
- Keep generated OpenAPI types as the source of truth for backend payloads.
- Avoid adding new `any` in production code. Test files have looser lint rules, but still prefer typed test helpers where practical.
- Keep hook state names consistent with generated React Query conventions: `isLoading`, `isPending`, `error`, `mutateAsync`.
- Use Zod or existing validation helpers for user input when validation belongs on the client.
- Keep browser API access guarded when code may run in tests or non-browser contexts.

## 7) Frontend React Standards

React conventions:

- Use function components and hooks.
- Keep page-level orchestration in `frontend/src/pages/`.
- Keep reusable layout and domain UI in `frontend/src/components/`.
- Keep API-facing behavior in hooks under `frontend/src/hooks/api/`.
- Use generated OpenAPI hooks from `frontend/src/openapi-rq/queries/queries` for backend calls.
- Use `backendFetch` only for manual authenticated fetch cases that cannot use generated clients.
- Keep React Query cache invalidation explicit after mutations.
- Prefer small, focused components over large pages with deeply mixed state, fetch, and rendering logic.
- Use Testing Library tests for user-visible behavior when component behavior changes.

## 8) UI and Styling Standards

The frontend uses Tailwind CSS and shadcn/ui conventions.

UI conventions:

- Reuse components from `frontend/src/components/ui/` before creating new primitives.
- Use `cn()` from `frontend/src/lib/utils.ts` for class composition.
- Keep variant styling in variant helper files when a component already uses that pattern.
- Use existing CSS variables and Tailwind tokens before introducing new colors.
- Keep layouts responsive for mobile and back-office use cases.
- Avoid inline styles unless a runtime value cannot be expressed cleanly through Tailwind/classes.
- Keep user-facing text localization-aware where the surrounding feature already uses i18n.

## 9) Testing Standards

Testing conventions:

- Add tests for new behavior and regressions.
- Keep tests deterministic and isolated.
- Mock real external providers and network calls.
- Use Testcontainers for backend database integration tests.
- Use `createQueryWrapper()` for frontend tests that need React Query.
- Use `MemoryRouter` for route-dependent frontend tests.
- Reset mocks, local storage, timers, and global stubs between tests.

See `docs/developer/testing.md` for detailed commands and CI mapping.

## 10) Security Standards

Security rules:

- Never commit real secrets, tokens, private keys, passwords, production URLs, or database dumps.
- Use `.env.example` files for placeholders only.
- Do not log passwords, OTPs, refresh tokens, access tokens, private keys, or provider credentials.
- Hash passwords with the existing password utility.
- Store refresh tokens as hashes, not raw token values.
- Use existing RBAC middleware instead of duplicating role checks ad hoc.
- Keep session/device state changes auditable where the domain requires it.
- Run Gitleaks when touching env files, deployment docs, or security-sensitive docs.

## 11) Commit and PR Standards

IVISS releases use Semantic Release, so commit messages should follow Conventional Commits:

- `feat: add organization activity export`
- `fix: handle expired refresh token`
- `docs: update developer testing guide`
- `chore: update dependencies`
- `refactor: simplify vehicle query mapping`
- `test: cover org admin user provisioning`

Breaking changes should use `!` or a `BREAKING CHANGE:` footer:

```text
feat!: change auth refresh contract
```

PR standards:

- Keep PRs focused.
- Include a short summary and validation commands.
- Mention migrations, generated artifacts, env var changes, and docs updates.
- Link related issues or task references when available.
- Do not include unrelated formatting or generated churn.

## 12) Pre-PR Checklist

Before opening a PR:

- Relevant backend and/or frontend checks pass.
- Tests cover the changed behavior.
- OpenAPI artifacts are regenerated when API contracts change.
- Migrations and seed data are synchronized when schema changes.
- No real secrets or local-only files are included.
- Developer docs are updated when commands, interfaces, workflows, or conventions change.
