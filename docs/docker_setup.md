# Frontend Docker Setup

This guide explains how to run the frontend application using Docker Compose.

## Prerequisites

- Docker Engine
- Docker Compose

## Local Development

To run the application in development mode with hot-reloading enabled:

1.  **Configure Environment**:
    ```bash
    cp frontend/.env.example frontend/.env
    ```

2.  **Start the Container**:
    ```bash
    docker compose up --build
    ```

The application will be accessible at: http://localhost:8080

**Features:**
- **Hot Reloading**: Changes to source files in `frontend/` are reflected immediately.
- **Port Mapping**: Host port 8080 is mapped to container port 8080.

## Production Build

To build and run the production-optimized container:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```

The application will be accessible at: http://localhost

**Features:**
- **Nginx Server**: Serves static assets efficiently.
- **Build Optimization**: Uses production build artifacts.
- **Immutable**: No volume mounts; changes require a rebuild.

## Environment Variables

Configuration is managed via `.env` files.

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_API_URL` | URL of the backend API | `http://localhost:3000/api` |
| `NODE_ENV` | Environment mode | `development` / `production` |

## Troubleshooting

- **Port Conflicts**: If port 8080 is in use, modify `docker-compose.override.yml`.
- **Node Modules**: If `node_modules` issues occur, rebuild without cache:
  ```bash
  docker compose build --no-cache
  ```
