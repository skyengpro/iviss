#!/bin/bash
set -e

# Load environment variables (to ensure DATABASE_URL is available if not using .env)
# But sqlx reads .env automatically.

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
echo "Login: System=PostgreSQL, Server=db, Username=postgres, Password=postgres, Database=iviss_dev"

