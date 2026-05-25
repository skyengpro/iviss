#!/bin/bash
set -e

echo "🧪 Running Backend Integration Tests"
echo "====================================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/iviss-backend"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Docker is running"

# Start database if not running
if ! docker compose -f "$PROJECT_ROOT/docker-compose.yml" ps db | grep -q "Up"; then
    echo -e "${YELLOW}⚠${NC}  Starting database..."
    docker compose -f "$PROJECT_ROOT/docker-compose.yml" up -d db
    echo "Waiting for database to be ready..."
    sleep 10
fi

echo -e "${GREEN}✓${NC} Database is ready"

# Set DATABASE_URL (using exposed port 5435)
export DATABASE_URL="${DATABASE_URL:-postgres://iviss_user:iviss_password@localhost:5435/iviss_dev}"

# Set required environment variables for tests
export SMS_PROVIDER="${SMS_PROVIDER:-vonage}"
export EMAIL_PROVIDER="${EMAIL_PROVIDER:-mock}"
export VONAGE_API_KEY="${VONAGE_API_KEY:-test_key}"
export VONAGE_API_SECRET="${VONAGE_API_SECRET:-test_secret}"

echo ""
echo "Running integration tests..."
echo "----------------------------"

cd "$BACKEND_DIR"

# Run tests (they're now in iviss-backend/tests/ following Cargo conventions)
if cargo test --test integration_* -- --nocapture; then
    echo ""
    echo -e "${GREEN}✓ Backend integration tests PASSED${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}✗ Backend integration tests FAILED${NC}"
    exit 1
fi
