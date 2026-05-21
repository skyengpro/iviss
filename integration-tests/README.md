# Integration Tests

Complete integration test suite for IVISS with backend, frontend, and E2E tests.

## Quick Commands

```bash
# Run all tests
./scripts/run-all-tests.sh

# Run specific suite
./scripts/run-backend-tests.sh
./scripts/run-frontend-tests.sh
./scripts/run-e2e-tests.sh
```

## Test Suites

### 1. Backend Integration Tests (15 tests)

**Location**: `backend/*.rs`

**What it tests**:
- HTTP request/response cycle through Axum
- Authentication endpoints (login, activation, daily login)
- Vehicle search endpoint validation
- Multi-tenant RBAC enforcement
- Control record CRUD operations

**Run**:
```bash
./scripts/run-backend-tests.sh
```

**Prerequisites**:
- Docker services running
- Database port exposed (see below)

### 2. Frontend Unit Tests (260 tests)

**Location**: `../frontend/src/**/__tests__/*.test.tsx`

**What it tests**:
- Authentication & state management
- API integration & interceptors
- Camera & OCR functionality
- Routing & navigation guards
- UI components
- Security & encryption

**Run**:
```bash
cd ../frontend
npm test
```

### 3. E2E Tests (18 tests)

**Location**: `e2e/tests/*.spec.ts`

**What it tests**:
- Admin user management workflow
- Agent device activation flow
- Multi-tenant data isolation

**Run**:
```bash
./scripts/run-e2e-tests.sh
```

**Prerequisites**:
- All Docker services running
- Playwright browsers installed

## Setup

### 1. Database Port Configuration

**For Security**: The database port is commented out in `docker-compose.yml` by default.

**To Run Tests**: Uncomment the port temporarily:

```yaml
# In docker-compose.yml
db:
  ports:
    - "5435:5432"  # Uncomment this line
```

**After Tests**: Comment it back:

```yaml
# In docker-compose.yml
db:
  ports:
    # - "5435:5432"  # Comment for security
```

### 2. Start Docker Services

```bash
# From project root
docker compose --profile dev up -d

# Verify services are healthy
docker compose ps
```

### 3. Install Dependencies

**Backend**: No additional dependencies (uses Cargo)

**Frontend**:
```bash
cd ../frontend
npm install
```

**E2E**:
```bash
cd e2e
npm install
npx playwright install chromium
```

## Running Tests

### All Tests at Once

```bash
./scripts/run-all-tests.sh
```

This will:
1. Run backend integration tests
2. Run frontend unit tests
3. Run E2E tests
4. Display summary

### Backend Tests Only

```bash
./scripts/run-backend-tests.sh
```

**What it does**:
- Sets environment variables
- Creates symlinks to test files
- Runs `cargo test --test integration_*`
- Shows results

### Frontend Tests Only

```bash
./scripts/run-frontend-tests.sh
```

**What it does**:
- Checks Node.js installation
- Installs dependencies if needed
- Runs `npm test`

### E2E Tests Only

```bash
./scripts/run-e2e-tests.sh
```

**What it does**:
- Checks Docker services
- Installs Playwright browsers
- Runs Playwright tests
- Generates HTML report

## Test Results

### Expected Output

```
✅ Backend Integration: 15/15 passing
✅ Frontend Unit: 260/260 passing
✅ E2E Tests: 18/18 passing (2 skipped)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Total: 293/293 tests passing (100%)
```

### View E2E Test Report

```bash
cd e2e
npx playwright show-report
```

## Troubleshooting

### Backend Tests: "Connection Refused"

**Problem**: Tests can't connect to database

**Solution**:
1. Uncomment database port in `docker-compose.yml`
2. Restart Docker services: `docker compose restart db`
3. Verify port is exposed: `docker compose ps`

### Backend Tests: "Password Authentication Failed"

**Problem**: Wrong database password

**Solution**:
- Check `.env` file has `POSTGRES_PASSWORD=iviss_password`
- Restart database: `docker compose restart db`

### E2E Tests: "Service Unavailable"

**Problem**: Docker services not running or not healthy

**Solution**:
```bash
# Check service status
docker compose ps

# Restart services
docker compose --profile dev down
docker compose --profile dev up -d

# Wait for health checks
docker compose ps  # All should show "healthy"
```

### E2E Tests: "Browser Not Found"

**Problem**: Playwright browsers not installed

**Solution**:
```bash
cd e2e
npx playwright install chromium
```

### Frontend Tests: Module Not Found

**Problem**: Dependencies not installed

**Solution**:
```bash
cd ../frontend
rm -rf node_modules package-lock.json
npm install
```

## Test Credentials

### Admin Users
- **Super Admin**: `admin@iviss.local` / `11111111`
- **Org Admin A**: `orgadmin1@gmail.com` / `11111111`
- **Org Admin B**: `orgadmin2@gmail.com` / `11111111`

### Agent Users
- **Agent 1 (Active)**: Badge `AGT-102`, Phone `+254700123457`
- **Agent 2 (Pending)**: Badge `AGT-104`, Phone `+237671210292`

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Start services
        run: docker compose --profile dev up -d
      
      - name: Run tests
        run: ./integration-tests/scripts/run-all-tests.sh
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: integration-tests/e2e/playwright-report/
```

## File Structure

```
integration-tests/
├── backend/                    # Backend integration tests
│   ├── integration_auth_flow.rs
│   ├── integration_database.rs
│   ├── integration_vehicle_search.rs
│   ├── integration_multi_tenant.rs
│   └── integration_control_records.rs
├── e2e/                        # E2E tests
│   ├── tests/
│   │   ├── admin-user-management.spec.ts
│   │   ├── agent-field-operation.spec.ts
│   │   └── multi-tenant-isolation.spec.ts
│   ├── playwright.config.ts
│   ├── package.json
│   └── README.md
├── scripts/                    # Test automation
│   ├── run-all-tests.sh
│   ├── run-backend-tests.sh
│   ├── run-frontend-tests.sh
│   └── run-e2e-tests.sh
└── README.md                   # This file
```

## Best Practices

### Before Committing
1. Run all tests: `./scripts/run-all-tests.sh`
2. Ensure 100% pass rate
3. Comment out database port in `docker-compose.yml`

### During Development
1. Keep database port uncommented for convenience
2. Run relevant test suite after changes
3. Fix failing tests before moving on

### In Production
1. Database port must be commented out
2. Use internal Docker networking only
3. Run tests in CI/CD pipeline

## Security Notes

### Database Port Exposure

**Development**: Uncomment port for testing convenience
```yaml
ports:
  - "5435:5432"  # OK for development
```

**Production**: Always comment out
```yaml
ports:
  # - "5435:5432"  # Commented for security
```

**Why**: Exposing the database port allows external connections, which is a security risk in production.

### Test Credentials

- Test credentials are in seed data (`iviss-backend/seeds/seed_data.sql`)
- Never use test credentials in production
- Change all passwords before deploying

---

**Quick Start Guide**: [../docs/INTEGRATION_TESTS.md](../docs/INTEGRATION_TESTS.md)  
**E2E Test Details**: [e2e/README.md](./e2e/README.md)
