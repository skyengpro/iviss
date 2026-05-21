# IVISS Integration Testing

Complete test suite for the IVISS project with backend integration tests, frontend unit tests, and end-to-end tests.

## Quick Start

```bash
# Run all tests
./integration-tests/scripts/run-all-tests.sh

# Run specific test suite
./integration-tests/scripts/run-backend-tests.sh
./integration-tests/scripts/run-frontend-tests.sh
./integration-tests/scripts/run-e2e-tests.sh
```

## Test Coverage

| Test Suite | Tests | Status |
|------------|-------|--------|
| Backend Integration | 15 tests | ✅ 100% |
| Frontend Unit | 260 tests | ✅ 100% |
| E2E Tests | 18 tests | ✅ 100% |
| **Total** | **293 tests** | **✅ 100%** |

## Prerequisites

### For Backend Tests
- Docker with database running
- Rust 1.70+
- **Important**: Uncomment database port in `docker-compose.yml`:
  ```yaml
  db:
    ports:
      - "5435:5432"  # Uncomment this line for tests
  ```

### For Frontend Tests
- Node.js 20+
- npm or yarn

### For E2E Tests
- Docker services running
- Node.js 20+
- Playwright browsers (auto-installed)

## Running Tests

### All Tests
```bash
./integration-tests/scripts/run-all-tests.sh
```

### Backend Integration Tests
```bash
# 1. Uncomment database port in docker-compose.yml
# 2. Start Docker services
docker compose --profile dev up -d

# 3. Run tests
./integration-tests/scripts/run-backend-tests.sh
```

### Frontend Unit Tests
```bash
cd frontend
npm test
```

### E2E Tests
```bash
# 1. Start Docker services
docker compose --profile dev up -d

# 2. Run E2E tests
./integration-tests/scripts/run-e2e-tests.sh
```

## Test Structure

```
integration-tests/
├── backend/              # Backend integration tests (Rust)
│   ├── integration_auth_flow.rs
│   ├── integration_vehicle_search.rs
│   ├── integration_multi_tenant.rs
│   └── integration_control_records.rs
├── e2e/                  # E2E tests (Playwright)
│   ├── tests/
│   │   ├── admin-user-management.spec.ts
│   │   ├── agent-field-operation.spec.ts
│   │   └── multi-tenant-isolation.spec.ts
│   └── playwright.config.ts
└── scripts/              # Test automation scripts
    ├── run-all-tests.sh
    ├── run-backend-tests.sh
    ├── run-frontend-tests.sh
    └── run-e2e-tests.sh
```

## What Each Test Suite Covers

### Backend Integration Tests (15 tests)
- ✅ HTTP request/response cycle
- ✅ Authentication endpoints
- ✅ Vehicle search validation
- ✅ Multi-tenant RBAC enforcement
- ✅ Control record CRUD operations

### Frontend Unit Tests (260 tests)
- ✅ Authentication & state management
- ✅ API integration & error handling
- ✅ Camera & OCR functionality
- ✅ Routing & navigation
- ✅ UI components
- ✅ Security & encryption

### E2E Tests (18 tests)
- ✅ Admin user management workflow
- ✅ Agent activation flow
- ✅ Multi-tenant data isolation

## Security Note

**Database Port**: The database port (`5435:5432`) is commented out in `docker-compose.yml` for security. Uncomment it only when running integration tests, then comment it back.

## Documentation

- **Quick Start**: [docs/INTEGRATION_TESTS.md](./docs/INTEGRATION_TESTS.md)
- **Complete Guide**: [integration-tests/README.md](./integration-tests/README.md)
- **E2E Tests**: [integration-tests/e2e/README.md](./integration-tests/e2e/README.md)

## Troubleshooting

### Backend Tests Fail with "Connection Refused"
- Ensure database port is uncommented in `docker-compose.yml`
- Check Docker services are running: `docker compose ps`

### E2E Tests Fail with "Service Unavailable"
- Ensure all Docker services are running
- Wait for services to be healthy: `docker compose ps`

### Frontend Tests Fail
- Clear node_modules: `rm -rf node_modules && npm install`
- Check Node.js version: `node --version` (should be 20+)

---

**Last Updated**: May 21, 2026  
**Test Pass Rate**: 100% (293/293 tests passing)
