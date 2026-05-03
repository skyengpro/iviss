# IVISS API Development Guide

This document explains how IVISS backend APIs are defined, documented, exposed, and consumed by the frontend.

## 1) API Contract Sources

The API contract is generated from the Rust backend.

Primary sources:

- `iviss-backend/src/routes.rs`: active Axum route registration and middleware grouping.
- `iviss-backend/src/api_doc.rs`: central OpenAPI document, tags, schemas, and security scheme.
- `iviss-backend/src/handlers/`: endpoint handlers and `#[utoipa::path]` annotations.
- `iviss-backend/src/dto/`: request and response DTOs exposed in OpenAPI.
- `iviss-backend/src/errors.rs`: shared API error response shape.

Generated or derived outputs:

- Runtime Swagger UI: `http://localhost:3000/docs`
- Runtime OpenAPI JSON: `http://localhost:3000/api-doc/openapi.json`
- Frontend snapshot: `frontend/openapi.json`
- Frontend generated client: `frontend/src/openapi-rq/`

Do not edit generated files under `frontend/src/openapi-rq/` by hand. Change the backend contract first, then regenerate the frontend client.

## 2) Route Groups

All current API routes use the `/api/v1` prefix.

### Public Routes

These routes do not require a bearer token:

| Method   | Path                                 | Purpose                                  |
| -------- | ------------------------------------ | ---------------------------------------- |
| `GET`  | `/api/v1/health`                   | Service health check                     |
| `POST` | `/api/v1/auth/login`               | Web/(super admin  and org admin) login |
| `POST` | `/api/v1/auth/activate`            | Device/account activation:               |
| `POST` | `/api/v1/auth/refresh`             | Refresh token request or challenge       |
| `POST` | `/api/v1/auth/refresh/verify`      | Refresh challenge verification           |
| `POST` | `/api/v1/auth/request-daily-login` | Request daily OTP login                  |
| `POST` | `/api/v1/auth/verify-daily-login`  | Verify daily OTP login                   |

### Web-Authenticated Routes

These routes concern the backoffice and require a valid web JWT:

| Method   | Path                             | Purpose                                                                  |
| -------- | -------------------------------- | ------------------------------------------------------------------------ |
| `POST` | `/api/v1/auth/change-password` | Change current user password at the first super admin or org admin login |
| `POST` | `/api/v1/auth/logout`          | Logout current session                                                   |

### Agent/Protected Routes

These routes require application authentication through `auth::require_auth`:

| Method   | Path                            | Purpose                                 |
| -------- | ------------------------------- | --------------------------------------- |
| `POST` | `/api/v1/scan/plate`          | Live scan OCR                           |
| `POST` | `/api/v1/photo/plate`         | Photo-based plate OCR                   |
| `POST` | `/api/v1/vehicles/search`     | Search vehicle and log lookup context   |
| `GET`  | `/api/v1/controls`            | List controls                           |
| `POST` | `/api/v1/controls`            | Create a control record                 |
| `POST` | `/api/v1/vehicles/pending`    | Submit pending vehicle/carte grise data |
| `GET`  | `/api/v1/stats/activity`      | Control activity stats                  |
| `GET`  | `/api/v1/stats/top-agents`    | Top agents stats                        |
| `GET`  | `/api/v1/stats/activity-feed` | Recent activity feed                    |
| `GET`  | `/api/v1/stats/recent-alerts` | Recent alerts                           |
| `GET`  | `/api/v1/users/me`            | Current user profile                    |
| `POST` | `/api/v1/users/location`      | Update current user location            |

### Admin Routes

These routes require web authentication and the admin RBAC middleware:

| Method     | Path                                     | Purpose                       |
| ---------- | ---------------------------------------- | ----------------------------- |
| `GET`    | `/api/v1/admin/submissions`            | List pending submissions      |
| `GET`    | `/api/v1/admin/submissions/:id`        | Get one pending submission    |
| `GET`    | `/api/v1/admin/submissions/:id/audit`  | Get submission audit log      |
| `GET`    | `/api/v1/admin/users`                  | List users                    |
| `POST`   | `/api/v1/admin/users`                  | Provision a user              |
| `GET`    | `/api/v1/admin/users/:id`              | Get a user                    |
| `PUT`    | `/api/v1/admin/users/:id`              | Update a user                 |
| `DELETE` | `/api/v1/admin/users/:id`              | Delete a user                 |
| `GET`    | `/api/v1/admin/organizations`          | List organizations            |
| `POST`   | `/api/v1/admin/organizations`          | Create organization           |
| `GET`    | `/api/v1/admin/organizations/:id`      | Get organization              |
| `PUT`    | `/api/v1/admin/organizations/:id`      | Update organization           |
| `DELETE` | `/api/v1/admin/organizations/:id`      | Delete organization           |
| `POST`   | `/api/v1/admin/terminate-session`      | Terminate user/device session |
| `POST`   | `/api/v1/admin/restart-session`        | Restart suspended session     |
| `POST`   | `/api/v1/admin/resend-activation-code` | Resend activation code        |
| `GET`    | `/api/v1/admin/controls/paged`         | Paginated controls list       |
| `GET`    | `/api/v1/admin/audit`                  | List audit logs               |
| `GET`    | `/api/v1/admin/audit/export`           | Export audit logs             |
| `GET`    | `/api/v1/admin/stats`                  | Admin dashboard stats         |

### Organization Admin Routes

These routes require web authentication and the org-admin RBAC middleware:

| Method   | Path                                | Purpose                                |
| -------- | ----------------------------------- | -------------------------------------- |
| `GET`  | `/api/v1/org-admin/users`         | List users in current organization     |
| `POST` | `/api/v1/org-admin/users`         | Provision user in current organization |
| `GET`  | `/api/v1/org-admin/stats`         | Organization dashboard stats           |
| `GET`  | `/api/v1/org-admin/activity-feed` | Organization activity feed             |
| `GET`  | `/api/v1/org-admin/recent-alerts` | Organization alerts                    |
| `GET`  | `/api/v1/org-admin/top-agents`    | Organization top agents                |
| `GET`  | `/api/v1/org-admin/activity`      | Organization control activity          |

## 3) Authentication and Error Shape

Protected OpenAPI operations use the `bearer_auth` security scheme:

```http
Authorization: Bearer <access_token>
```

The generated frontend client attaches tokens through `frontend/src/services/auth/authInterceptor.ts`.
Manual fetch helpers should use `frontend/src/services/api/backendFetch.ts` when they need authenticated backend requests.

Errors returned through `AppError` follow this JSON shape:

```json
{
  "code": "UNAUTHORIZED",
  "message": "Invalid token"
}
```

Known error codes are:

- `UNAUTHORIZED`
- `FORBIDDEN`
- `DATABASE_ERROR`
- `NOT_FOUND`
- `BAD_REQUEST`
- `TOO_MANY_REQUESTS`
- `EXTERNAL_API_FAILURE`
- `INTERNAL_ERROR`

Use typed `AppError` variants in handlers and service boundaries instead of returning ad hoc JSON errors.

## 4) Adding or Changing an Endpoint

Use this order when changing the backend API:

1. Define or update DTOs in `iviss-backend/src/dto/`.
2. Add or update the handler in `iviss-backend/src/handlers/`.
3. Add or update query/service logic in `iviss-backend/src/queries/` or `iviss-backend/src/services/`.
4. Register the route in `iviss-backend/src/routes.rs`.
5. Add the handler path and schemas to `iviss-backend/src/api_doc.rs` if needed.
6. Add or update tests in `iviss-backend/src/tests/`.
7. Refresh the OpenAPI spec and regenerate the frontend client.
8. Update consuming hooks/services in `frontend/src/hooks/` or `frontend/src/services/`.

When adding `#[utoipa::path]` annotations:

- Use the same path and method as `routes.rs`.
- Add the correct OpenAPI tag from `api_doc.rs`.
- Include `security(("bearer_auth" = []))` on protected endpoints.
- Reference DTO schemas instead of inline untyped JSON.
- Document expected success and error responses.

## 5) OpenAPI and Frontend Codegen

The frontend uses generated request functions, types, and React Query hooks from `frontend/src/openapi-rq/`.

Important generated files:

- `requests/services.gen.ts`: generated request functions.
- `requests/types.gen.ts`: generated TypeScript types.
- `queries/queries.ts`: generated React Query hooks.
- `queries/common.ts`: generated query keys and shared query helpers.

Typical frontend usage:

```ts
import { useSearchVehicleV1 } from '@/openapi-rq/queries/queries';
import type { VehicleSearchRequest } from '@/openapi-rq/requests/types.gen';
```

Regenerate the frontend client from the current `frontend/openapi.json`:

```bash
cd frontend
npm run codegen
```

Fetch the OpenAPI spec from a running backend before codegen:

```bash
cd frontend
BACKEND_OPENAPI_URL=http://127.0.0.1:3000/api-doc/openapi.json npm run predev
npm run codegen
```

Export the OpenAPI JSON directly from backend code:

```bash
cd iviss-backend
cargo run --bin export_openapi
```

The `frontend` dev server also runs `predev`, which attempts to refresh `frontend/openapi.json` from the backend and falls back to the local snapshot if the backend is unavailable.

## 6) Local Verification

Start the stack:

```bash
docker compose --profile dev up -d --build
```

Check health:

```bash
curl http://localhost:3000/api/v1/health
```

Open Swagger UI:

```text
http://localhost:3000/docs
```

Fetch the OpenAPI JSON:

```bash
curl http://localhost:3000/api-doc/openapi.json
```

Run relevant checks after API changes:

```bash
cd iviss-backend
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

```bash
cd frontend
npm run codegen
npm run ts:check
npm run test
```

## 7) API Change Checklist

Before opening a PR that changes an API:

- The route exists in `routes.rs`.
- The OpenAPI path annotation matches the active route.
- Request and response DTOs are represented in `api_doc.rs`.
- Protected routes declare `bearer_auth`.
- Errors use the shared `AppErrorResponse` shape.
- Backend tests cover the new or changed behavior.
- `frontend/openapi.json` and `frontend/src/openapi-rq/` are regenerated when the frontend contract changes.
- Frontend hooks/services compile against the regenerated types.
- Developer documentation is updated if endpoint behavior, auth, or codegen workflow changes.
