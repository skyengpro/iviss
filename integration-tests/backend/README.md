# Integration Tests

This directory contains integration tests for the IVISS backend.

## Test Structure

- `integration_auth_flow.rs` - Tests authentication endpoints (login, refresh, logout)
- `integration_database.rs` - Tests database operations with testcontainers
- `integration_vehicle_search.rs` - Tests vehicle search and external API integration (TODO)
- `integration_multi_tenant.rs` - Tests multi-tenant data isolation (TODO)

## Running Integration Tests

### Run all tests (unit + integration)
```bash
cargo test
```

### Run only integration tests
```bash
cargo test --test integration_*
```

### Run specific integration test file
```bash
cargo test --test integration_auth_flow
```

### Run with output
```bash
cargo test --test integration_auth_flow -- --nocapture
```

### Run with database
```bash
# Make sure DATABASE_URL is set
export DATABASE_URL="postgres://iviss_user:password@localhost:5432/iviss_dev"
cargo test
```

### Run with Docker (testcontainers)
```bash
# Testcontainers will automatically start PostgreSQL
cargo test --test integration_database
```

## Environment Variables

- `DATABASE_URL` - Required for tests that need database access
- `SKIP_DOCKER_TESTS` - Set to skip tests that require Docker
- `RUST_LOG` - Set to `debug` or `trace` for verbose output

## CI/CD Integration

These tests run automatically in GitHub Actions via `.github/workflows/backend-ci.yml`.

The CI pipeline:
1. Starts PostgreSQL service
2. Runs migrations
3. Executes all tests with coverage
4. Generates coverage report

## Writing New Integration Tests

1. Create a new file: `tests/integration_<feature>.rs`
2. Use `#[tokio::test]` for async tests
3. Use `setup_test_state()` helper for app state
4. Use testcontainers for database tests
5. Add cleanup logic in test teardown

Example:
```rust
#[tokio::test]
async fn test_my_feature() {
    let state = setup_test_state().await;
    // Your test logic here
}
```

## Best Practices

- ✅ Test real HTTP requests through the router
- ✅ Use testcontainers for database isolation
- ✅ Clean up test data after each test
- ✅ Use meaningful test names
- ✅ Test error cases, not just happy paths
- ❌ Don't rely on shared state between tests
- ❌ Don't use production database for tests
- ❌ Don't skip cleanup on test failure
