# Docker Setup

This guide explains how to run the application using Docker Compose.

## Prerequisites

- Docker Engine
- Docker Compose

## Development (Local)

The default services in `docker-compose.yml` are development-oriented, with hot reloading enabled for both Frontend and Backend.

**Start the stack:**

```bash
docker compose up --build
```

- **Frontend**: http://localhost:8080
- **Backend**: http://localhost:3000
- **Database**: Port 5435

**Features:**

- **Hot Reloading**: Source code changes are reflected immediately.
- **Data Persistence**: Database data is stored in the `postgres_data` volume.
- **Adminer**: Available in local development on `http://localhost:8081`.

## Production (CI/CD)

Production images are built and pushed to GitHub Container Registry (GHCR) automatically via GitHub Actions.

- **Workflow**: `.github/workflows/docker-publish.yml`
- **Triggers**: Pushes to `main` and `dev` branches.
- **Images**:
  - `ghcr.io/<owner>/iviss/frontend`
  - `ghcr.io/<owner>/iviss/backend`

To run the production-like stack locally from this compose file, use the dedicated prod services:

```bash
docker compose --profile prod up --build db redis backend-prod frontend-prod metrics
```

- `backend-prod` uses the backend `production` target.
- `frontend-prod` uses the frontend `prod` target.
- No source-code bind mounts are applied to these prod services.

## Troubleshooting

- **Port Conflicts**: Ensure ports 8080, 3000, 5435, 6380, and 8081 are free.
- **Rebuild**: If dependencies change, run `docker compose up --build`.
