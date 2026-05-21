#!/bin/bash
set -e

echo "🧪 Running Frontend Integration Tests"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FRONTEND_DIR="$PROJECT_ROOT/frontend"

echo "Checking Node.js installation..."
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Node.js $(node --version)"

echo ""
echo "Installing dependencies..."
echo "-------------------------"

cd "$FRONTEND_DIR"

if [ ! -d "node_modules" ]; then
    echo "Installing npm packages..."
    npm ci --legacy-peer-deps
fi

echo -e "${GREEN}✓${NC} Dependencies installed"

echo ""
echo "Running tests..."
echo "----------------"

# Run tests
if npm run test -- --run; then
    echo ""
    echo -e "${GREEN}✓ Frontend tests PASSED${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}✗ Frontend tests FAILED${NC}"
    exit 1
fi
