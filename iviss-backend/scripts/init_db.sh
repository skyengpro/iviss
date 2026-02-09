#!/usr/bin/env bash
set -eo pipefail

# --- check dependencies --- 
# if ! [ -x "$(command -v psql)" ]; then
#   echo >&2 "❌ Error: psql is not installed."
#   exit 1
# fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "❌ Error: sqlx is not installed."
  echo >&2 "👉 Use:"
  echo >&2 " cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
  exit 1
fi

# Configuration
DB_USER="${POSTGRES_USER:=iviss}"
DB_PASSWORD="${POSTGRES_PASSWORD:=sky_for_iviss}"
DB_NAME="${POSTGRES_DB:=iviss_db}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"
CONTAINER_NAME="iviss-postgres-rs"

# --- remove old container if exists ---
if [ "$(docker ps -aq -f name=${CONTAINER_NAME})" ]; then
    echo "🧹 Suppression de l'ancien conteneur..."
    docker rm -f ${CONTAINER_NAME}
fi

# --- run PG databse Docker ---
if [[ -z "${SKIP_DOCKER}" ]]
then
  echo "🚀 Starting PostgreSQL container with Docker..."
  docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    --name ${CONTAINER_NAME} \
    -d postgres:17-alpine \
    postgres -N 1000
fi

# --- waiting postgres to be ready ---
export PGPASSWORD="${DB_PASSWORD}"

echo "⏳ Waiting for PostgreSQL to become available..."
# until docker exec $(docker ps -q --filter name=${CONTAINER_NAME}) \
#   psql -U "${DB_USER}" -d "${DB_NAME}" -c '\q' 2>/dev/null; do
until docker exec ${CONTAINER_NAME} pg_isready -U ${DB_USER} > /dev/null 2>&1; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done
echo "✅ Postgres is up and running on port ${DB_PORT}!"

# --- integration and Configuration ---
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

echo "🛠️  Creating database if not exists..."
sqlx database create

echo "📦 Running database migrations..."
sqlx migrate run

echo "✅PostgreSQL has been migrated successfully and is ready to go!"
