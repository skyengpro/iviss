# IVISS Debugging Guide

This guide lists common local debugging workflows for backend, frontend, and integration issues.

## 1) First Response Checklist

When something fails locally:

1. Confirm services are up:

```bash
docker compose ps
```

2. Check backend and frontend logs:

```bash
docker compose logs -f backend
docker compose logs -f frontend
```

3. Validate backend health:

```bash
curl http://localhost:3000/api/v1/health
```

4. Confirm OpenAPI is reachable:

```bash
curl http://localhost:3000/api-doc/openapi.json
```

## 2) Backend Startup Failures

Common symptoms:

- Container exits at startup
- `Config`/env validation errors
- Migration failures

Checks:

```bash
docker compose logs --tail=200 backend
```

Typical root causes:

- Missing required env vars (`JWT_PRIVATE_KEY_PEM`, `JWT_PUBLIC_KEY_PEM`, `SMS_PROVIDER`, `ACTIVATION_CODE_PEPPER`)
- Invalid provider credentials for selected `SMS_PROVIDER`
- Invalid DB connection values

## 3) Database and Migration Issues

Symptoms:

- `Database error`
- Migration apply errors
- Query behavior mismatch after schema updates

Checks:

```bash
docker compose logs --tail=200 db
docker compose logs --tail=200 backend
```

Use Adminer for quick verification:

- `http://localhost:8081`

Rules:

- Never edit already-applied migrations on shared branches.
- Add a new migration for every schema change.

## 4) Authentication and RBAC Issues

Symptoms:

- `401 UNAUTHORIZED`
- `403 FORBIDDEN`
- Unexpected route access denial

Checks:

- Verify the token is attached by frontend interceptor.
- Confirm route group and middleware in `iviss-backend/src/routes.rs`.
- Confirm role mapping in backend (`admin`, `manager`, `org_admin`, `agent`).

Useful files:

- `iviss-backend/src/middleware/auth.rs`
- `iviss-backend/src/middleware/rbac.rs`
- `frontend/src/services/auth/authInterceptor.ts`

## 5) OpenAPI and Codegen Drift

Symptoms:

- Frontend TypeScript errors after API changes
- Missing generated hooks/types

Fix sequence:

```bash
cd frontend
BACKEND_OPENAPI_URL=http://127.0.0.1:3000/api-doc/openapi.json npm run predev
npm run codegen
npm run ts:check
```

If backend is not running, `predev` falls back to `frontend/openapi.json`.

## 6) Frontend Runtime Issues

Symptoms:

- Blank page / navigation loop
- API calls failing in browser
- Role-based page access mismatch

Checks:

```bash
docker compose logs --tail=200 frontend
```

Also check:

- Router guards in `frontend/src/router/RequireAuth.tsx`
- Role values returned by backend user profile endpoint
- Browser network tab for failing `/api` requests

## 7) Test Failures

Backend:

```bash
cd iviss-backend
cargo test -- --nocapture
```

Frontend:

```bash
cd frontend
npm run test
```

If tests rely on generated API types, regenerate code first.

## 8) Useful Reset Actions (Local Only)

Full local stack restart:

```bash
docker compose down
docker compose --profile dev up -d --build
```

Reset local volumes (destructive):

```bash
docker compose down -v
docker compose --profile dev up -d --build
```

Use destructive resets only for local development environments.
