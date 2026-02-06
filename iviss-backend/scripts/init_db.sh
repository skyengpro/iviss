#!/usr/bin/env bash
set -euo pipefail

# Configuration
DB_USER="${POSTGRES_USER:-iviss}"
DB_PASSWORD="${POSTGRES_PASSWORD:-iviss_dev_password}"
DB_NAME="${POSTGRES_DB:-iviss_internal}"
DB_PORT="${POSTGRES_PORT:-5432}"
DB_HOST="${POSTGRES_HOST:-localhost}"

