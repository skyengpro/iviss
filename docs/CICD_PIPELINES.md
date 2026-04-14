# CI/CD Pipelines - Technical Documentation

**Document Version:** 1.0  
**Last Updated:** April 2026  
**Target Audience:** DevOps/Deployment Team

---

## Overview

IVISS uses **GitHub Actions** for continuous integration and container image publishing. The CI/CD setup is **partially implemented** - it handles testing and image building but **does not include deployment automation**.

### Pipeline Maturity Assessment

| Stage               | Status             | Notes                              |
| ------------------- | ------------------ | ---------------------------------- |
| Source Control      | ✅ Implemented     | GitHub repository                  |
| Code Quality        | ✅ Implemented     | Linting, formatting, type checking |
| Testing             | ✅ Implemented     | Unit tests, integration tests      |
| Security Scanning   | ✅ Implemented     | Gitleaks, cargo audit              |
| Build               | ✅ Implemented     | Docker multi-stage builds          |
| Artifact Publishing | ✅ Implemented     | GHCR (GitHub Container Registry)   |
| Deployment          | ❌ Not Implemented | No automated deployment            |
| Smoke Tests         | ❌ Not Implemented | No post-deployment validation      |
| Rollback            | ❌ Not Implemented | No automated rollback              |

---

## CI/CD Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         GitHub Repository                        │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         │ Push/PR Event
                         │
         ┌───────────────┴───────────────┐
         │                               │
         ▼                               ▼
┌─────────────────┐            ┌─────────────────┐
│  Backend CI     │            │  Frontend CI    │
│  Workflow       │            │  Workflow       │
└────────┬────────┘            └────────┬────────┘
         │                               │
         │                               │
    ┌────┴────┐                     ┌────┴────┐
    │         │                     │         │
    ▼         ▼                     ▼         ▼
  Build    Test                  Build    Test
  Lint     Audit                 Lint     Coverage
  Format   Coverage              Format   SonarQube
  Doc                            TypeCheck
         │                               │
         └───────────────┬───────────────┘
                         │
                         ▼
                ┌─────────────────┐
                │ Docker Publish  │
                │   Workflow      │
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │      GHCR       │
                │  (Container     │
                │   Registry)     │
                └─────────────────┘
                         │
                         │ [DEPLOYMENT GAP]
                         │ No automation beyond this point
                         ▼
                    (Manual)
```

---

## Workflow Files

### 1. Backend CI Pipeline

**File:** `.github/workflows/backend-ci.yml`

**Triggers:**

- Push to `main` or `dev` branches (when backend files change)
- Pull requests to `main` or `dev` (when backend files change)
- Path filters: `iviss-backend/**`, `.github/workflows/backend-ci.yml`

**Jobs:**

#### Job 1: Gitleaks Scan

- **Purpose:** Detect secrets in code
- **Tool:** Gitleaks Docker image
- **Config:** `.gitleaks.toml`
- **Runs on:** ubuntu-latest
- **Execution:** On every trigger

#### Job 2: Backend Build

- **Purpose:** Verify code compiles
- **Dependencies:** System packages (clang, mold, tesseract, leptonica)
- **Rust Toolchain:** Stable (via actions-rust-lang/setup-rust-toolchain)
- **Command:** `cargo build --verbose`
- **Working Directory:** `iviss-backend/`

#### Job 3: Backend Test & Coverage

- **Purpose:** Run tests with coverage reporting
- **Dependencies:** backend_build job
- **Tools:**
  - cargo-llvm-cov (coverage)
  - cargo-nextest (test runner)
- **Steps:**
  1. Install system dependencies
  2. Set up Rust toolchain
  3. Cache cargo-llvm-cov binary
  4. Install cargo-nextest
  5. Clean coverage artifacts
  6. Run tests with coverage instrumentation
  7. Run doc tests
  8. Generate HTML coverage report
  9. Fail if coverage < 60%
  10. Upload coverage artifact (3-day retention)

**Coverage Threshold:** 60% line coverage (enforced)

#### Job 4: Backend Format

- **Purpose:** Enforce code formatting
- **Tool:** rustfmt
- **Command:** `cargo fmt --all -- --check`
- **Failure:** Pipeline fails if code is not formatted

#### Job 5: Backend Clippy

- **Purpose:** Lint code for common mistakes
- **Tool:** clippy
- **Command:** `cargo clippy -- -D warnings`
- **Failure:** Pipeline fails on any warnings

#### Job 6: Backend Audit

- **Purpose:** Check for security vulnerabilities in dependencies
- **Tool:** cargo-audit
- **Ignored Advisories:**
  - RUSTSEC-2023-0071 (rsa Marvin Attack - transitive dependency)
  - RUSTSEC-2025-0111 (tokio-tar - dev dependency)
- **Failure:** Pipeline fails on unignored vulnerabilities

#### Job 7: Backend Documentation

- **Purpose:** Verify documentation builds
- **Command:** `cargo doc --no-deps --verbose`
- **Failure:** Pipeline fails if docs don't build

**Total Jobs:** 7  
**Parallel Execution:** Yes (except coverage depends on build)  
**Average Duration:** ~5-10 minutes

---

### 2. Frontend CI Pipeline

**File:** `.github/workflows/frontend-ci.yml`

**Triggers:**

- Push to `main` or `dev` branches (when frontend files change)
- Pull requests to `main` or `dev` (when frontend files change)
- Path filters: `frontend/**`, `.github/workflows/frontend-ci.yml`

**Jobs:**

#### Job 1: Frontend Install

- **Purpose:** Install npm dependencies and cache
- **Node Version:** 20
- **Command:** `npm ci --legacy-peer-deps`
- **Cache:** node_modules (key: OS + package-lock.json hash)
- **Working Directory:** `frontend/`

#### Job 2: OpenAPI Codegen

- **Purpose:** Generate TypeScript API client from OpenAPI spec
- **Dependencies:** frontend_install job
- **Command:** `npm run codegen`
- **Output:** `frontend/src/openapi-rq/`
- **Cache:** Generated files (key: commit SHA)

#### Job 3: Frontend Build

- **Purpose:** Verify production build succeeds
- **Dependencies:** frontend_install, openapi_codegen
- **Command:** `npm run build`
- **Output:** `frontend/dist/`

#### Job 4: Frontend Lint

- **Purpose:** ESLint code quality checks
- **Dependencies:** frontend_install, openapi_codegen
- **Command:** `npm run lint:check`
- **Failure:** Pipeline fails on lint errors

#### Job 5: Frontend Prettier

- **Purpose:** Enforce code formatting
- **Dependencies:** frontend_install, openapi_codegen
- **Command:** `npm run prettier:check`
- **Failure:** Pipeline fails if code is not formatted

#### Job 6: Frontend TypeScript

- **Purpose:** Type checking
- **Dependencies:** frontend_install, openapi_codegen
- **Command:** `npm run ts:check`
- **Failure:** Pipeline fails on type errors

#### Job 7: Frontend Unit Tests

- **Purpose:** Run tests with coverage
- **Dependencies:** frontend_install, openapi_codegen
- **Command:** `npm run coverage -- --reporter=verbose`
- **Output:** `frontend/coverage/`
- **Cache:** Coverage report (key: commit SHA)

#### Job 8: Frontend SonarQube Analysis

- **Purpose:** Code quality and security analysis
- **Dependencies:** frontend_build, frontend_unit_tests
- **Tool:** SonarQube
- **Secrets Required:**
  - `SONAR_TOKEN`
  - `SONAR_HOST_URL`
- **Config:**
  - Project Key: IVISS
  - Sources: `src/`
  - Coverage: `coverage/lcov.info`
  - Quality Gate: Wait for result

**Total Jobs:** 8  
**Parallel Execution:** Yes (with dependency graph)  
**Average Duration:** ~8-12 minutes

---

### 3. Docker Publish Pipeline

**File:** `.github/workflows/docker-publish.yml`

**Triggers:**

- Push to `main` or `dev` branches
- Pull requests to `main` or `dev`
- Manual workflow dispatch (requires confirmation)

**Manual Trigger:**

- Input: `confirm` (must type "yes")
- Purpose: Allow manual image builds

**Strategy:**

- **Matrix Build:** Builds frontend and backend in parallel
- **Services:** `[frontend, backend]`

**Matrix Configuration:**

| Service  | Context           | Dockerfile Target |
| -------- | ----------------- | ----------------- |
| frontend | `./frontend`      | `prod`            |
| backend  | `./iviss-backend` | `production`      |

**Jobs:**

#### Job: Build and Push

**Permissions:**

- `contents: read` - Read repository
- `packages: write` - Push to GHCR

**Steps:**

1. **Checkout Repository**
   - Action: `actions/checkout@v4`

2. **Set up Docker Buildx**
   - Action: `docker/setup-buildx-action@v3`
   - Purpose: Enable advanced Docker build features

3. **Log in to Container Registry**
   - Action: `docker/login-action@v3`
   - Registry: `ghcr.io`
   - Username: `${{ github.actor }}`
   - Password: `${{ secrets.GITHUB_TOKEN }}`

4. **Extract Metadata**
   - Action: `docker/metadata-action@v5`
   - Purpose: Generate image tags and labels
   - **Tags Generated:**
     - `sha-<full-commit-sha>` (e.g., `sha-abc123...`)
     - `<branch-name>` (e.g., `main`, `dev`)
     - `latest`

5. **Build and Push Image**
   - Action: `docker/build-push-action@v5`
   - **Build Args:**
     - Context: Matrix-specific (frontend or backend)
     - Target: Matrix-specific (prod or production)
   - **Push Condition:**
     - Push on: Direct push to branches
     - Skip on: Pull requests (build only, no push)
     - Push on: Manual workflow dispatch
   - **Cache:**
     - Type: GitHub Actions cache
     - Mode: max (cache all layers)

**Image Naming Convention:**

```
ghcr.io/<owner>/<repo>/frontend:latest
ghcr.io/<owner>/<repo>/frontend:main
ghcr.io/<owner>/<repo>/frontend:sha-<commit-sha>

ghcr.io/<owner>/<repo>/backend:latest
ghcr.io/<owner>/<repo>/backend:main
ghcr.io/<owner>/<repo>/backend:sha-<commit-sha>
```

**Total Jobs:** 1 (with 2 parallel matrix builds)  
**Average Duration:** ~10-15 minutes

---

## Branch Strategy

### Current Branch Workflow

```
main (production-ready)
  │
  ├─ CI/CD runs on push
  ├─ Docker images tagged: main, latest, sha-xxx
  │
dev (development)
  │
  ├─ CI/CD runs on push
  ├─ Docker images tagged: dev, sha-xxx
  │
feature/* (feature branches)
  │
  ├─ CI/CD runs on PR to main/dev
  ├─ No Docker images pushed (build only)
```

### Branch Protection (Recommended - Not Configured)

The deployment team should configure:

- [ ] Require PR reviews before merge
- [ ] Require status checks to pass
- [ ] Require branches to be up to date
- [ ] Restrict who can push to main
- [ ] Require signed commits

---

## Secrets Management in CI/CD

### Currently Used Secrets

| Secret           | Used In            | Purpose          | Rotation     |
| ---------------- | ------------------ | ---------------- | ------------ |
| `GITHUB_TOKEN`   | docker-publish.yml | Push to GHCR     | Auto-managed |
| `SONAR_TOKEN`    | frontend-ci.yml    | SonarQube auth   | Manual       |
| `SONAR_HOST_URL` | frontend-ci.yml    | SonarQube server | Manual       |

### Missing Secrets (Required for Deployment)

When deployment is implemented, these will be needed:

- [ ] Production database credentials
- [ ] Production Redis credentials
- [ ] JWT private/public keys (production)
- [ ] Twilio credentials (production)
- [ ] SSL/TLS certificates
- [ ] Cloud provider credentials (AWS, Azure, GCP)
- [ ] Kubernetes cluster credentials (if applicable)
- [ ] Deployment webhook URLs

**Recommendation:** Use GitHub Environments for secret management per environment (dev/staging/prod).

---

## Artifact Management

### Container Images

**Registry:** GitHub Container Registry (GHCR)  
**Visibility:** Private (requires authentication)  
**Retention:** No automatic cleanup configured

**Image Sizes (Approximate):**

- Backend (production): ~150-200 MB
- Frontend (production): ~50-80 MB

**Image Layers:**

- Backend: Debian slim + Tesseract + compiled Rust binary
- Frontend: Nginx alpine + static assets

### Build Artifacts

**Coverage Reports:**

- Backend: HTML report (3-day retention)
- Frontend: lcov.info (used by SonarQube)

**Recommendation:** Implement artifact cleanup policy to avoid storage costs.

---

## Testing Strategy

### Backend Testing

**Test Types:**

- Unit tests (via cargo test)
- Integration tests (via cargo nextest)
- Doc tests (via cargo test --doc)

**Test Execution:**

- Tool: cargo-nextest (faster than cargo test)
- Coverage: cargo-llvm-cov
- Threshold: 60% line coverage

**Test Environment:**

- Database: Not used (tests use mocks or testcontainers)
- Redis: Not used (tests use mocks)

### Frontend Testing

**Test Types:**

- Unit tests (Vitest)
- Component tests (React Testing Library)

**Test Execution:**

- Tool: Vitest
- Coverage: Istanbul (via Vitest)
- Reporter: Verbose + lcov

**Test Environment:**

- Browser: jsdom (simulated)
- API: Mocked

### Missing Test Types

- [ ] End-to-end tests (Playwright, Cypress)
- [ ] Performance tests (k6, JMeter)
- [ ] Security tests (OWASP ZAP, Burp Suite)
- [ ] Load tests
- [ ] Smoke tests (post-deployment)

---

## Deployment Stages (NOT IMPLEMENTED)

### Recommended Future Pipeline

```
┌─────────────┐
│   Commit    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  CI Tests   │ ✅ Implemented
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Build Image │ ✅ Implemented
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Push to     │ ✅ Implemented
│   GHCR      │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Deploy to   │ ❌ NOT IMPLEMENTED
│   Dev Env   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Smoke Tests │ ❌ NOT IMPLEMENTED
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Deploy to   │ ❌ NOT IMPLEMENTED
│ Staging Env │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Integration │ ❌ NOT IMPLEMENTED
│   Tests     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Manual    │ ❌ NOT IMPLEMENTED
│  Approval   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Deploy to   │ ❌ NOT IMPLEMENTED
│  Production │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Health      │ ❌ NOT IMPLEMENTED
│  Checks     │
└─────────────┘
```

---

## Failure Handling

### Current Behavior

**On CI Failure:**

- Pipeline stops
- GitHub shows red X on commit/PR
- No notifications configured

**On Build Failure:**

- Image is not pushed to GHCR
- Previous images remain available

**On Test Failure:**

- Pipeline fails
- Coverage report still uploaded (if tests ran)

### Missing Failure Handling

- [ ] Slack/email notifications on failure
- [ ] Automatic rollback on deployment failure
- [ ] Incident creation on production failure
- [ ] Automatic retry logic
- [ ] Canary deployment rollback

---

## Performance Optimization

### Current Optimizations

✅ **Implemented:**

- Docker layer caching (GitHub Actions cache)
- Cargo dependency caching
- npm dependency caching
- Parallel job execution
- Matrix builds for frontend/backend

### Potential Improvements

- [ ] Self-hosted runners (faster than GitHub-hosted)
- [ ] Build cache warming
- [ ] Incremental builds
- [ ] Artifact reuse across jobs
- [ ] Conditional job execution (skip if no changes)

---

## Monitoring & Observability

### CI/CD Metrics (Available in GitHub)

- Build duration
- Success/failure rate
- Test pass rate
- Coverage trends

### Missing CI/CD Observability

- [ ] Deployment frequency metrics
- [ ] Lead time for changes
- [ ] Mean time to recovery (MTTR)
- [ ] Change failure rate
- [ ] Deployment success rate

**Recommendation:** Implement DORA metrics tracking.

---

## Security in CI/CD

### Current Security Measures

✅ **Implemented:**

- Gitleaks secret scanning
- Cargo audit (dependency vulnerabilities)
- SonarQube code analysis
- Minimal container images (alpine/slim)
- Non-root user in production containers

### Security Gaps

- [ ] Container image scanning (Trivy, Snyk)
- [ ] SBOM (Software Bill of Materials) generation
- [ ] Signed container images (Cosign)
- [ ] Admission control (OPA, Kyverno)
- [ ] Runtime security monitoring

---

## Rollback Strategy (NOT IMPLEMENTED)

### Current State

**No rollback mechanism exists.**

If a bad deployment occurs:

1. Manual intervention required
2. Revert code changes
3. Re-run CI/CD pipeline
4. Wait for new images to build
5. Manually deploy previous version

### Recommended Rollback Strategy

1. **Image Tagging:**
   - Keep last N production images
   - Tag with semantic version + commit SHA

2. **Deployment Automation:**
   - Store deployment history
   - One-click rollback to previous version
   - Automatic database migration rollback (if safe)

3. **Canary Deployments:**
   - Deploy to small percentage of traffic
   - Monitor metrics
   - Auto-rollback on error spike

---

## Cost Optimization

### Current Costs

**GitHub Actions:**

- Free tier: 2,000 minutes/month (public repos)
- Current usage: ~50-100 minutes per push
- Estimated monthly usage: ~2,000-4,000 minutes

**GitHub Container Registry:**

- Free tier: 500 MB storage (public repos)
- Current usage: ~500 MB - 1 GB
- Bandwidth: Free for public repos

**Potential Costs:**

- If private repo: $0.008/minute after free tier
- If exceeding storage: $0.25/GB/month

### Cost Optimization Recommendations

- [ ] Implement image cleanup policy (delete old images)
- [ ] Use self-hosted runners for high-frequency builds
- [ ] Cache more aggressively
- [ ] Skip CI on documentation-only changes

---

## Handover Checklist for Deployment Team

### Immediate Actions

1. **Review Existing Pipelines**
   - [ ] Run all workflows manually to verify they work
   - [ ] Check for any failing jobs
   - [ ] Review coverage reports

2. **Configure Branch Protection**
   - [ ] Set up required status checks
   - [ ] Configure PR review requirements
   - [ ] Restrict direct pushes to main

3. **Set Up Notifications**
   - [ ] Configure Slack/email alerts for failures
   - [ ] Set up on-call rotation
   - [ ] Document escalation procedures

4. **Plan Deployment Stages**
   - [ ] Design deployment workflow
   - [ ] Choose deployment tool (ArgoCD, Flux, custom scripts)
   - [ ] Define environment promotion strategy
   - [ ] Implement approval gates

5. **Implement Missing Tests**
   - [ ] Add end-to-end tests
   - [ ] Add smoke tests for post-deployment
   - [ ] Add performance tests
   - [ ] Add security tests

6. **Set Up Monitoring**
   - [ ] Track DORA metrics
   - [ ] Monitor pipeline performance
   - [ ] Set up cost tracking
   - [ ] Implement deployment dashboards

---

## Troubleshooting Guide

### Common Issues

**Issue:** Backend build fails with "tesseract not found"

- **Cause:** System dependencies not installed
- **Fix:** Ensure `libtesseract-dev` is in apt-get install step

**Issue:** Frontend build fails with "openapi-rq not found"

- **Cause:** Codegen step didn't run
- **Fix:** Ensure openapi_codegen job completed successfully

**Issue:** Docker push fails with "authentication required"

- **Cause:** GITHUB_TOKEN expired or insufficient permissions
- **Fix:** Check workflow permissions in repository settings

**Issue:** Coverage job fails with "coverage below threshold"

- **Cause:** Test coverage dropped below 60%
- **Fix:** Add more tests or adjust threshold

**Issue:** SonarQube job fails with "quality gate failed"

- **Cause:** Code quality issues detected
- **Fix:** Review SonarQube report and fix issues

---

## Future Enhancements

### Short Term (1-3 months)

1. Add deployment automation to dev environment
2. Implement smoke tests
3. Set up deployment notifications
4. Add container image scanning
5. Implement DORA metrics tracking

### Medium Term (3-6 months)

1. Add staging environment deployment
2. Implement canary deployments
3. Add end-to-end tests
4. Set up automatic rollback
5. Implement blue-green deployments

### Long Term (6-12 months)

1. Multi-region deployment
2. Advanced monitoring and observability
3. Chaos engineering
4. Self-healing infrastructure
5. GitOps with ArgoCD/Flux

---

## References

- GitHub Actions Documentation: https://docs.github.com/en/actions
- Docker Build Documentation: https://docs.docker.com/build/
- GHCR Documentation: https://docs.github.com/en/packages
- DORA Metrics: https://cloud.google.com/blog/products/devops-sre/using-the-four-keys-to-measure-your-devops-performance

---

## Document Revision History

| Version | Date       | Author          | Changes                     |
| ------- | ---------- | --------------- | --------------------------- |
| 1.0     | April 2026 | DevOps Analysis | Initial CI/CD documentation |
