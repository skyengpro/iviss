#!/bin/bash
set -e

echo "🧪 IVISS Integration Test Suite"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Track results
BACKEND_EXIT=0
FRONTEND_EXIT=0
E2E_EXIT=0

echo -e "${BLUE}Starting test suite...${NC}"
echo ""

# Run backend tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  1/3: Backend Integration Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/run-backend-tests.sh"; then
    BACKEND_EXIT=0
else
    BACKEND_EXIT=1
fi

echo ""
echo ""

# Run frontend tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  2/3: Frontend Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/run-frontend-tests.sh"; then
    FRONTEND_EXIT=0
else
    FRONTEND_EXIT=1
fi

echo ""
echo ""

# Run E2E tests (if they exist)
if [ -f "$SCRIPT_DIR/run-e2e-tests.sh" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  3/3: End-to-End Tests"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if bash "$SCRIPT_DIR/run-e2e-tests.sh"; then
        E2E_EXIT=0
    else
        E2E_EXIT=1
    fi
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  3/3: End-to-End Tests"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}⚠${NC}  E2E tests not yet implemented"
    E2E_EXIT=0
fi

echo ""
echo ""

# Print summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test Results Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ $BACKEND_EXIT -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} Backend Integration Tests: PASSED"
else
    echo -e "  ${RED}✗${NC} Backend Integration Tests: FAILED"
fi

if [ $FRONTEND_EXIT -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} Frontend Tests: PASSED"
else
    echo -e "  ${RED}✗${NC} Frontend Tests: FAILED"
fi

if [ -f "$SCRIPT_DIR/run-e2e-tests.sh" ]; then
    if [ $E2E_EXIT -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} End-to-End Tests: PASSED"
    else
        echo -e "  ${RED}✗${NC} End-to-End Tests: FAILED"
    fi
else
    echo -e "  ${YELLOW}⊘${NC} End-to-End Tests: NOT IMPLEMENTED"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Exit with error if any test failed
if [ $BACKEND_EXIT -ne 0 ] || [ $FRONTEND_EXIT -ne 0 ] || [ $E2E_EXIT -ne 0 ]; then
    echo ""
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All tests passed!${NC}"
exit 0
