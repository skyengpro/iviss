#!/bin/bash
set -e

echo "🎭 E2E Test Setup Script"
echo "========================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${BLUE}Step 1: Checking prerequisites...${NC}"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker is not installed${NC}"
    echo "Please install Docker: https://docs.docker.com/get-docker/"
    exit 1
fi
echo -e "${GREEN}✓${NC} Docker is installed"

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}✗ Docker is not running${NC}"
    echo "Please start Docker and try again"
    exit 1
fi
echo -e "${GREEN}✓${NC} Docker is running"

# Check Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}✗ Node.js is not installed${NC}"
    echo "Please install Node.js 20+: https://nodejs.org/"
    exit 1
fi
NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo -e "${YELLOW}⚠${NC}  Node.js version is $NODE_VERSION, recommend 20+"
else
    echo -e "${GREEN}✓${NC} Node.js $(node -v) is installed"
fi

echo ""
echo -e "${BLUE}Step 2: Starting IVISS services...${NC}"
echo ""

cd "$PROJECT_ROOT"

# Check if services are already running
if docker compose ps frontend | grep -q "Up"; then
    echo -e "${GREEN}✓${NC} Services are already running"
else
    echo "Starting services with Docker Compose..."
    docker compose --profile dev up -d
    
    echo ""
    echo "Waiting for services to be ready..."
    echo -n "Backend: "
    
    # Wait for backend (max 120 seconds)
    COUNTER=0
    until curl -sf http://localhost:3000/api/v1/health > /dev/null 2>&1; do
        if [ $COUNTER -gt 60 ]; then
            echo -e "${RED}✗ Timeout${NC}"
            echo "Backend did not start in time. Check logs:"
            echo "  docker compose logs backend"
            exit 1
        fi
        echo -n "."
        sleep 2
        COUNTER=$((COUNTER + 1))
    done
    echo -e " ${GREEN}✓${NC}"
    
    echo -n "Frontend: "
    # Wait for frontend (max 120 seconds)
    COUNTER=0
    until curl -sf http://localhost:8080 > /dev/null 2>&1; do
        if [ $COUNTER -gt 60 ]; then
            echo -e "${RED}✗ Timeout${NC}"
            echo "Frontend did not start in time. Check logs:"
            echo "  docker compose logs frontend"
            exit 1
        fi
        echo -n "."
        sleep 2
        COUNTER=$((COUNTER + 1))
    done
    echo -e " ${GREEN}✓${NC}"
fi

echo ""
echo -e "${BLUE}Step 3: Installing E2E test dependencies...${NC}"
echo ""

cd "$SCRIPT_DIR"

if [ ! -d "node_modules" ]; then
    echo "Installing npm packages..."
    npm install
    echo -e "${GREEN}✓${NC} Dependencies installed"
else
    echo -e "${GREEN}✓${NC} Dependencies already installed"
fi

echo ""
echo -e "${BLUE}Step 4: Installing Playwright browsers...${NC}"
echo ""

if [ ! -d "$HOME/.cache/ms-playwright" ]; then
    echo "Installing Playwright Chromium browser..."
    npx playwright install chromium
    echo -e "${GREEN}✓${NC} Playwright browser installed"
else
    echo -e "${GREEN}✓${NC} Playwright browsers already installed"
fi

echo ""
echo -e "${BLUE}Step 5: Checking test data...${NC}"
echo ""

# Check if database has users
USER_COUNT=$(docker exec iviss-db psql -U iviss_user -d iviss_dev -t -c "SELECT COUNT(*) FROM users;" 2>/dev/null | xargs || echo "0")

if [ "$USER_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Database has $USER_COUNT user(s)"
else
    echo -e "${YELLOW}⚠${NC}  Database has no users"
    echo ""
    echo "You need to create test users before running E2E tests."
    echo "See E2E_TESTING_GUIDE.md Step 3 for instructions."
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✓ E2E Test Setup Complete!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo ""
echo "1. Create test users (if not already done):"
echo "   - See E2E_TESTING_GUIDE.md Step 3"
echo "   - Or login to http://localhost:8080 as admin"
echo ""
echo "2. Run E2E tests:"
echo "   ${BLUE}cd $SCRIPT_DIR${NC}"
echo "   ${BLUE}npx playwright test${NC}"
echo ""
echo "3. View results:"
echo "   ${BLUE}npx playwright show-report${NC}"
echo ""
echo "For more options, see:"
echo "   ${BLUE}cat E2E_TESTING_GUIDE.md${NC}"
echo ""
