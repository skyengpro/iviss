# IVISS

IVISS is a multi-tenant platform for law-enforcement and regulatory teams to run roadside controls, verify vehicle compliance, and manage enforcement workflows.

## Table of Contents

- [Overview](#overview)
- [Architecture at a Glance](#architecture-at-a-glance)
- [Tech Stack](#tech-stack)
- [Quick Start (Local Development)](#quick-start-local-development)
- [Useful Endpoints](#useful-endpoints)
- [Run Frontend Outside Docker (Optional)](#run-frontend-outside-docker-optional)
- [Quality Checks](#quality-checks)
- [Project Structure](#project-structure)
- [Documentation Map](#documentation-map)
- [Production Infrastructure](#production-infrastructure)
- [Contributing](#contributing)
- [License](#license)

## Overview

IVISS helps agencies:

- Identify vehicles (manual entry, photo/OCR workflows).
- Perform field control operations and keep traceable control history.
- Trigger and track enforcement actions.
- Operate with role-based access control in a multi-organization environment.
- Use a mobile-first web experience with PWA capabilities.

## Architecture at a Glance

- **Frontend**: React + TypeScript + Vite (mobile-first SPA, PWA-enabled)
- **Backend**: Rust + Axum + SQLx (REST API + OpenAPI)
- **Database**: PostgreSQL 15 (Docker)
- **Infra**: Docker Compose for local, Terraform + Ansible + GitHub Actions for production

## Tech Stack

### Frontend

- React, TypeScript, Vite
- Tailwind CSS + shadcn/ui (Radix primitives)
- TanStack Query, React Router
- Vitest + Testing Library

### Backend

- Rust, Axum, Tokio
- SQLx (PostgreSQL)
- Utoipa + Swagger UI (OpenAPI generation and docs)

### Platform

- Docker / Docker Compose
- GitHub Actions CI/CD
- AWS Lightsail (deployment target)

## Quick Start (Local Development)

### Prerequisites

- Docker Engine 20.10+
- Docker Compose v2+
- Node.js 20+ (only if running frontend outside Docker)

### 1) Configure environment

```bash
cp .env.example .env
```

Set required values in `.env` before starting containers.
At minimum for local boot:

- `POSTGRES_PASSWORD`
- `EXTERNAL_POSTGRES_PASSWORD`
- `JWT_PRIVATE_KEY_PEM`
- `JWT_PUBLIC_KEY_PEM`
- `ACTIVATION_CODE_PEPPER`
- `SMS_PROVIDER`

Recommended for end-to-end back-office flows:

- `EMAIL_PROVIDER` (`mock`, `resend`, `lettre`/`smtp`)
- Provider credentials for the selected email mode (for example SMTP or Resend keys)

Note:

- The backend can start with `EMAIL_PROVIDER=mock` (or unset, which defaults to `mock`).
- To provision an organization admin and deliver the temporary password by real email, configure a real provider (`resend` or `lettre`/`smtp`) with valid credentials.

### 2) Start the development stack

```bash
docker compose --profile dev up -d --build
```

This starts the local development services (database, backend, frontend, adminer, metrics).

### 3) Check service status and logs

```bash
docker compose ps
docker compose logs -f backend
docker compose logs -f frontend
```

### 4) Stop services

```bash
# Keep volumes
docker compose down

# Remove volumes (destructive for local DB data)
docker compose down -v
```

## Useful Endpoints

- Frontend (dev): http://localhost:8080
- Backend health: http://localhost:3000/api/v1/health
- Swagger UI: http://localhost:3000/docs
- OpenAPI JSON: http://localhost:3000/api-doc/openapi.json
- Adminer: http://localhost:8081
- Metrics service health: http://localhost:9091/health

## Run Frontend Outside Docker (Optional)

If you prefer running the frontend directly on Node:

```bash
cd frontend
npm install
npm run dev
```

Notes:

- Dev server runs on `http://localhost:8080`.
- `predev` fetches OpenAPI from backend (`http://127.0.0.1:3000/api-doc/openapi.json`) and falls back to local `frontend/openapi.json` if unavailable.

## Quality Checks

### Frontend

```bash
cd frontend
npm run lint
npm run ts:check
npm run test
npm run build
```

### Backend

```bash
cd iviss-backend
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Project Structure

```text
iviss/
├── .github/         # CI/CD workflows
├── docs/            # Product, architecture, and operational documentation
├── frontend/        # React + TypeScript application
├── iviss-backend/   # Rust + Axum API service
├── infra/           # Terraform + Ansible infrastructure code
├── monitoring/      # Monitoring-related assets
└── scripts/         # Utility scripts (e.g., OpenAPI fetch)
```

## Documentation Map

- System overview: [docs/overview.md](docs/overview.md)
- Technical architecture: [docs/architecture_spec.md](docs/architecture_spec.md)
- Data model / schema: [docs/schema.md](docs/schema.md)
- Deployment and operations: [docs/deployment_guide.md](docs/deployment_guide.md)
- Monitoring guide: [docs/monitoring.md](docs/monitoring.md)
- PWA testing guide: [docs/pwa_testing_guide.md](docs/pwa_testing_guide.md)
  #### Developer onboarding guide
- Developer documentation index: [docs/developer/README.md](docs/developer/README.md)
- Developer getting started: [docs/developer/getting-started.md](docs/developer/getting-started.md)
- Developer project structure: [docs/developer/project-structure.md](docs/developer/project-structure.md)
- Developer API guide: [docs/developer/api.md](docs/developer/api.md)
- Developer database guide: [docs/developer/database.md](docs/developer/database.md)
- Developer testing guide: [docs/developer/testing.md](docs/developer/testing.md)
- Developer coding standards: [docs/developer/coding-standards.md](docs/developer/coding-standards.md)
- Developer debugging guide: [docs/developer/debugging.md](docs/developer/debugging.md)


## Production Infrastructure

[![Deployment Status](https://github.com/skyengpro/iviss/actions/workflows/deploy-aws.yml/badge.svg)](https://github.com/skyengpro/iviss/actions/workflows/deploy-aws.yml)

IVISS production deployments are managed with Infrastructure-as-Code and CI/CD.

- **Target**: AWS Lightsail
- **Provisioning**: Terraform
- **Configuration/Deploy**: Ansible + Docker Compose
- **Pipelines**: GitHub Actions

For full production setup, secrets, and runbooks, see:
[docs/deployment_guide.md](docs/deployment_guide.md)

## Contributing

- Use dedicated feature branches.
- Keep PRs focused and reviewable.
- Update documentation when behavior, configuration, or interfaces change.
- Ensure local checks pass before opening a PR.

## License

This project is proprietary. All rights reserved.
