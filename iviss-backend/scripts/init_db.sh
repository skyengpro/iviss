#!/bin/bash
set -e

# Load environment variables (to ensure DATABASE_URL is available if not using .env)
# But sqlx reads .env automatically.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [ -f "$ROOT_DIR/.env" ]; then
    set -a
    . "$ROOT_DIR/.env"
    set +a
fi

if [ -f "$ROOT_DIR/iviss-backend/.env" ]; then
    set -a
    . "$ROOT_DIR/iviss-backend/.env"
    set +a
fi

if [ -z "${DATABASE_URL:-}" ]; then
    POSTGRES_USER_VALUE="${POSTGRES_USER:-iviss_user}"
    if [ -z "${POSTGRES_PASSWORD:-}" ]; then
        echo "POSTGRES_PASSWORD must be set in .env before running init_db.sh" >&2
        exit 1
    fi
    POSTGRES_PASSWORD_VALUE="${POSTGRES_PASSWORD}"
    POSTGRES_DB_VALUE="${POSTGRES_DB:-iviss_db}"
    DATABASE_URL="postgres://${POSTGRES_USER_VALUE}:${POSTGRES_PASSWORD_VALUE}@localhost:5435/${POSTGRES_DB_VALUE}"
    export DATABASE_URL
fi

# Start docker containers
echo "Starting Database & Adminer..."
docker compose up -d

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "sqlx could not be found. Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Wait for database to be ready (simple sleep for now, could use pg_isready)
echo "Waiting for database to start..."
sleep 5

# Create Database and Run Migrations
echo "Creating database & running migrations..."
sqlx database create
sqlx migrate run

echo "setup complete!"
echo "You can visualize the database at http://localhost:8081"
echo "Login: System=PostgreSQL, Server=db, Username=${POSTGRES_USER:-iviss_user}, Password=<your local POSTGRES_PASSWORD>, Database=${POSTGRES_DB:-iviss_db}"
