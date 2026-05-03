# IVISS Developer Getting Started

This guide helps you run IVISS locally and validate a healthy development setup.

## Prerequisites

- Docker Engine `20.10+`
- Docker Compose `v2+`
- Node.js `20+` (only if running frontend outside Docker)

## 1) Prepare Environment Variables

From repository root:

```bash
cp .env.example .env
```

Required values for backend startup:

- `POSTGRES_PASSWORD`
- `JWT_PRIVATE_KEY_PEM`
- `JWT_PUBLIC_KEY_PEM`
- `ACTIVATION_CODE_PEPPER` (minimum 32 chars)
- `SMS_PROVIDER` (`vonage`, `twilio`, or `orange`)
- `EMAIL_PROVIDER` (lettre)

Provider-specific required variables:

- If `SMS_PROVIDER=vonage`: set `VONAGE_API_KEY`, `VONAGE_API_SECRET`
- If `SMS_PROVIDER=twilio`: set `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_FROM_NUMBER`
- If `SMS_PROVIDER=orange`: set `ORANGE_CLIENT_ID`, `ORANGE_CLIENT_SECRET`
- If `EMAIL_PROVIDER`=lettre: set `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_FROM_EMAIL`

Notes:

- `EMAIL_PROVIDER` defaults to `mock` if unset.
- `ADMIN_BOOTSTRAP_*` variables are optional. If all are set, an admin seed is attempted at startup.
- `EXTERNAL_POSTGRES_PASSWORD` and other `EXTERNAL_POSTGRES_...` variable for the moment are not yet use because we don't have the external database credentials yet.

## 2) Start the Development Stack

From repository root:

```bash
docker compose --profile dev up -d --build
```

This starts local development services (database, backend, frontend, adminer, metrics).

## 3) Verify the Stack

Check containers:

```bash
docker compose ps
```

Check backend health:

```bash
curl http://localhost:3000/api/v1/health
```

Expected response:

```text
OK
```

Useful local URLs:

- Frontend: `http://localhost:8080`
- Swagger UI: `http://localhost:3000/docs`
- OpenAPI JSON: `http://localhost:3000/api-doc/openapi.json`
- Adminer: `http://localhost:8081`
- Metrics health: `http://localhost:9091/health`

## 4) Useful Runtime Commands

Follow logs:

```bash
docker compose logs -f backend
docker compose logs -f frontend
```

Restart a service:

```bash
docker compose restart backend
```

Stop stack:

```bash
docker compose down
```

Stop stack and remove volumes (destructive for local DB data):

```bash
docker compose down -v
```

## 5) Optional: Run Frontend Outside Docker

If backend is already running on `localhost:3000`:

```bash
cd frontend
npm install
npm run dev
```

Notes:

- Frontend runs on `http://localhost:8080`.
- On `npm run dev`, the `predev` hook fetches OpenAPI from `http://127.0.0.1:3000/api-doc/openapi.json` and falls back to `frontend/openapi.json` if backend is unreachable.

## 6) Optional: Run Backend Outside Docker

First you need to import the backend .env file

```bash
cp iviss-backend/.env.example iviss-backend/.env
```

And then run the cargo run command:

```bash
cd iviss-backend
cargo build
cargo run
```

Note:

* Backend run on `http://localhost:`3000
* `SHIFT_START_HOUR` and `SHIFT_END_HOUR` are not longer use for the moment as we can set this shift time when creating and organization directly

## 7) Common Startup Issues

`SMS_PROVIDER must be set`:

- Set `SMS_PROVIDER` in `.env` with one of `vonage|twilio|orange`.

`Invalid SMS_PROVIDER value`:

- Use only `vonage`, `twilio`, or `orange`.

`JWT_PRIVATE_KEY_PEM must be set` or `JWT_PUBLIC_KEY_PEM must be set`:

- Ensure both PEM variables are present and non-empty in `.env`.

`ACTIVATION_CODE_PEPPER must be at least 32 characters`:

- Use a longer value in `.env`.

Backend unhealthy after startup:

- Check logs with `docker compose logs -f backend`.
- Confirm DB is healthy with `docker compose ps`.
