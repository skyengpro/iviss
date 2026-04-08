# Frontend Build and CI Workflow Fix Documentation

This document outlines the modifications made to resolve the frontend build failures and restore the CI/CD pipeline functionality.

## 1. Problem Description

The frontend application encountered two primary issues during the build and codegen processes:
- **Deprecated Dependency**: The previous OpenAPI generator (`@7nohe/openapi-react-query-codegen`) was failing with "Variable declaration not found" errors under modern Node.js environments.
- **Resource Deletion during Codegen**: The codegen tool was configured to clean the `src/openapi-rq` directory, which inadvertently deleted manually created compatibility layers needed for the legacy code.
- **Node.js Version Mismatch**: Modern dependencies (Vite 7, @hey-api) required Node.js 20+, while some environments were still targeting Node 18.

## 2. Implemented Solutions

### A. Codegen Migration
The project has been migrated to the modern and actively maintained `@hey-api/openapi-ts` tool with the `@tanstack/react-query` plugin.

**Changes:**
- Updated `frontend/package.json` with new dependencies and scripts.
- Created `frontend/openapi-ts.config.ts` for unified configuration.

### B. Architectural Restructuring
To prevent automated tools from breaking the codebase, the API client output was restructured:
- **Managed Directory**: All generated code now resides in `src/openapi-rq/generated/`. Automated codegen only cleans this sub-directory.
- **Stable Compatibility Layer**: Manually managed re-exports live in `src/openapi-rq/queries/` and `src/openapi-rq/requests/`. These files are **not** managed by the codegen and will persist across runs.

### C. CI/CD Workflow Fixes
The Github Actions workflow was updated to ensure build stability.

**Fixes:**
- Standardized Node.js version to `20` (minimum supported by Vite 7).
- Verified caching mechanisms for the restructured `src/openapi-rq` path.

## 3. Maintenance Guide

### Syncing with Backend Changes
If the backend OpenAPI specification changes, follow these steps to update the frontend:

1.  **Export Spec**: From `iviss-backend`, run:
    ```bash
    cargo run --bin export_openapi > ../frontend/openapi.json
    ```
2.  **Generate Client**: From `frontend`, run:
    ```bash
    npm run codegen
    ```
3.  **Verify**: Run `npm run build` to ensure all type-safe hooks and services are correctly integrated.
