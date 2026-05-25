# Backend Integration Tests - MOVED

**⚠️ IMPORTANT: Backend integration tests have been moved!**

## New Location

All backend integration tests are now located in:
```
iviss-backend/tests/
```

This follows Rust/Cargo conventions where integration tests should live in the `tests/` directory next to `Cargo.toml`.

## Running Tests

To run backend integration tests:

```bash
cd iviss-backend
cargo test
```

Or run specific tests:

```bash
cd iviss-backend
cargo test integration_auth_flow
cargo test integration_vehicle_search
cargo test integration_multi_tenant
cargo test integration_control_records
```

## Test Structure

```
iviss-backend/tests/
├── helpers/
│   └── mod.rs                          # Shared test utilities
├── integration_auth_flow.rs            # Authentication flow tests
├── integration_vehicle_search.rs       # Vehicle search tests
├── integration_multi_tenant.rs         # Multi-tenant isolation tests
└── integration_control_records.rs      # Control record CRUD tests
```

## See Also

- Main integration tests documentation: `docs/INTEGRATION_TESTS.md`
- Backend-specific tests: `iviss-backend/tests/`
- E2E tests: `integration-tests/e2e/`
