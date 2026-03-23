# Docker Setup

This guide explains how to run the application using Docker Compose.

## Prerequisites

- Docker Engine
- Docker Compose

## Development (Local)

The `docker-compose.yml` file is configured for local development by default, with hot-reloading enabled for both Frontend and Backend.

**Start the stack:**

```bash
docker compose up --build
```

- **Frontend**: http://localhost:8080
- **Backend**: http://localhost:3000
- **Database**: Port 5432

**Features:**

- **Hot Reloading**: Source code changes are reflected immediately.
- **Data Persistence**: Database data is stored in the `postgres_data` volume.

## Production (CI/CD)

Production images are built and pushed to GitHub Container Registry (GHCR) automatically via GitHub Actions.

- **Workflow**: `.github/workflows/docker-publish.yml`
- **Triggers**: Pushes to `main` and `dev` branches.
- **Images**:
  - `ghcr.io/<owner>/iviss/frontend`
  - `ghcr.io/<owner>/iviss/backend`

To run production locally, you would need to use `docker compose -f docker-compose.yml` and override the `build.target` context or pull the images from the registry.

## Troubleshooting

- **Port Conflicts**: Ensure ports 8080, 3000, and 5432 are free.
- **Rebuild**: If dependencies change, run `docker compose up --build`.
