#!/bin/bash
set -e

echo "🎭 Running E2E Tests with Playwright"
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
E2E_DIR="$PROJECT_ROOT/integration-tests/e2e"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Docker is running"

# Check if services are running
if ! docker compose -f "$PROJECT_ROOT/docker-compose.yml" ps frontend | grep -q "Up"; then
    echo -e "${YELLOW}⚠${NC}  Services not running. Starting services..."
    docker compose -f "$PROJECT_ROOT/docker-compose.yml" --profile dev up -d
    echo "Waiting for services to be ready..."
    sleep 30
fi

echo -e "${GREEN}✓${NC} Services are running"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed${NC}"
    echo "Please install Node.js to run E2E tests"
    exit 1
fi

echo -e "${GREEN}✓${NC} Node.js is installed"

# Navigate to E2E directory
cd "$E2E_DIR"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo ""
    echo "Installing E2E test dependencies..."
    echo "-----------------------------------"
    npm install
    
    echo ""
    echo "Installing Playwright browsers..."
    echo "--------------------------------"
    npx playwright install chromium
fi

echo ""
echo "Running E2E tests..."
echo "-------------------"

# Set base URL
export BASE_URL="${BASE_URL:-http://localhost:8080}"

# Run Playwright tests
if npx playwright test; then
    echo ""
    echo -e "${GREEN}✓ E2E tests PASSED${NC}"
    echo ""
    echo "To view the test report, run:"
    echo "  cd integration-tests/e2e && npx playwright show-report"
    exit 0
else
    echo ""
    echo -e "${RED}✗ E2E tests FAILED${NC}"
    echo ""
    echo "To view the test report, run:"
    echo "  cd integration-tests/e2e && npx playwright show-report"
    exit 1
fi
