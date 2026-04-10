# Container Architecture - Technical Documentation

**Document Version:** 1.0  
**Last Updated:** April 2026  
**Target Audience:** DevOps/Deployment Team

---

## Overview

IVISS uses Docker containers orchestrated by Docker Compose for local development. The application consists of 7 containerized services with a microservices-style architecture. All containers run on a single Docker bridge network with no external orchestration (no Kubernetes, Docker Swarm, or similar).

### Containerization Maturity: **Level 2 - Docker Compose**

- ✅ All services containerized
- ✅ Multi-stage Docker builds
- ✅ Development and production build targets
- ✅ Health checks configured
- ✅ Volume persistence
- ❌ No container orchestration platform
- ❌ No horizontal scaling
- ❌ No service mesh
- ❌ No container security scanning

---

## Container Inventory

### Service Overview

| # | Service | Image | Purpose | Exposed Ports | Status |
|---|---------|-------|---------|---------------|--------|
| 1 | db | postgres:15-alpine | Primary database | 5435→5432 | Production-ready |
| 2 | redis | redis:7-alpine | Cache & session store | 6380→6379 | Production-ready |
| 3 | backend | Custom (dev) | API server (hot-reload) | 3000→3000 | Development only |
| 4 | backend-prod | Custom (prod) | API server (optimized) | 3000→3000 | Production-ready |
| 5 | frontend | Custom (dev) | React app (Vite dev server) | 8080→8080 | Development only |
| 6 | frontend-prod | Custom (prod) | React app (Nginx) | 8080→80 | Production-ready |
| 7 | adminer | adminer:latest | Database admin UI | 8081→8080 | Development only |
| 8 | metrics | Custom | Prometheus metrics collector | 9091→9091 | Production-ready |

**Total Services:** 8 (6 run by default, 2 opt-in with `--profile prod`)

---

## Service Details

### 1. PostgreSQL Database (db)

**Image:** `postgres:15-alpine`  
**Container Name:** `iviss-db`  
**Purpose:** Primary application database

**Configuration:**

```yaml
Environment Variables:
  POSTGRES_USER: iviss_user (default)
  POSTGRES_PASSWORD: Required (no default)
  POSTGRES_DB: iviss_dev (default)

Ports:
  Host: 5435
  Container: 5432

Volumes:
  postgres_data:/var/lib/postgresql/data (persistent)

Health Check:
  Command: pg_isready -U iviss_user -d iviss_dev
  Interval: 10s
  Timeout: 5s
  Retries: 5
  Start Period: 30s

Restart Policy: unless-stopped

Resource Limits: None configured

Logging:
  Driver: json-file
  Max Size: 10m
  Max Files: 3
```

**Dependencies:** None (base service)

**Data Persistence:** Yes (postgres_data volume)

**Backup Strategy:** ❌ Not implemented

**Production Readiness:** ✅ Ready (standard PostgreSQL image)

---

### 2. Redis Cache (redis)

**Image:** `redis:7-alpine`  
**Container Name:** `iviss-redis`  
**Purpose:** Cache, OTP storage, rate limiting, session data

**Configuration:**
```yaml
Ports:
  Host: 6380
  Container: 6379

Volumes:
  redis_data:/data (persistent)

Health Check:
  Command: redis-cli ping
  Interval: 10s
  Timeout: 5s
  Retries: 5
  Start Period: 10s

Restart Policy: unless-stopped

Resource Limits:
  CPU: 0.25 cores (limit), 0.1 cores (reservation)
  Memory: 256MB (limit), 64MB (reservation)

Logging:
  Driver: json-file
  Max Size: 10m
  Max Files: 3
```

**Dependencies:** None (base service)

**Data Persistence:** Yes (redis_data volume)

**Production Readiness:** ✅ Ready

**Notes:**
- Only service with resource limits configured
- Persistence enabled (RDB snapshots)
- No Redis password configured (⚠️ security gap)

---

### 3. Backend (Development)

**Image:** Custom (built from `iviss-backend/Dockerfile`, target: `development`)  
**Container Name:** `iviss-backend`  
**Purpose:** API server with hot-reload for development

**Base Image:** `rust:1.89-slim-bookworm`

**Build Configuration:**
```dockerfile
Target: development
Base: rust:1.89-slim-bookworm
System Dependencies:
  - pkg-config
  - libtesseract-dev
  - libleptonica-dev
  - tesseract-ocr-eng
  - clang
  - curl
  - ca-certificates
Additional Tools:
  - cargo-watch (for hot-reload)
```

**Runtime Configuration:**
```yaml
Ports:
  Host: 3000
  Container: 3000

Environment Variables:
  DATABASE_URL: postgres://iviss_user:***@iviss-db:5432/iviss_dev
  EXTERNAL_DATABASE_URL: postgres://external_user:***@localhost:5432/external_db
  REDIS_URL: redis://iviss-redis:6379
  SQLX_OFFLINE: "true"
  JWT_PRIVATE_KEY_PEM: (from .env)
  JWT_PUBLIC_KEY_PEM: (from .env)
  ACTIVATION_CODE_PEPPER: (from .env)
  ENVIRONMENT: local
  TWILIO_ACCOUNT_SID: mock (default)
  TWILIO_AUTH_TOKEN: mock (default)
  TWILIO_FROM_NUMBER: mock (default)
  SERVER_HOST: 0.0.0.0
  SERVER_PORT: 3000
  RUST_LOG: info (default)
  ADMIN_BOOTSTRAP_EMAIL: (from .env)
  ADMIN_BOOTSTRAP_PASSWORD: (from .env)
  ADMIN_BOOTSTRAP_PHONE: (from .env)
  ADMIN_BOOTSTRAP_USERNAME: (from .env)

DNS Servers:
  - 1.1.1.1
  - 8.8.8.8

Volumes (Read-Only Source Mounts):
  - ./iviss-backend/src:/app/src:ro
  - ./iviss-backend/Cargo.toml:/app/Cargo.toml:ro
  - ./iviss-backend/Cargo.lock:/app/Cargo.lock:ro
  - ./iviss-backend/migrations:/app/migrations:ro
  - ./iviss-backend/.sqlx:/app/.sqlx:ro
  - ./iviss-backend/seeds:/app/seeds:ro

Volumes (Build Caches):
  - cargo_cache:/usr/local/cargo/registry
  - target_cache:/app/target

Health Check:
  Command: curl -f http://localhost:3000/api/v1/health || exit 1
  Interval: 10s
  Timeout: 5s
  Retries: 5
  Start Period: 180s (3 minutes - allows for initial compilation)

Restart Policy: unless-stopped

Command: cargo watch -w src -w migrations -x run
```

**Dependencies:**
- db (must be healthy)
- redis (must be healthy)

**Hot Reload:** ✅ Enabled (cargo-watch monitors src/ and migrations/)

**Production Readiness:** ❌ Development only

**Image Size:** ~2-3 GB (includes full Rust toolchain)

---

### 4. Backend (Production)

**Image:** Custom (built from `iviss-backend/Dockerfile`, target: `production`)  
**Container Name:** Not named (profile-based)  
**Purpose:** Optimized API server for production

**Base Image:** `debian:bookworm-slim`

**Build Strategy:**
```dockerfile
Stage 1 (base): Install system dependencies
Stage 2 (development): Warm cache (not used in prod)
Stage 3 (builder): Multi-stage build
  - Build dependencies first (cached layer)
  - Build application (separate layer)
  - Output binary to /out/iviss-backend
Stage 4 (production): Minimal runtime
  - Debian slim base
  - Runtime dependencies only
  - Non-root user (iviss:iviss)
  - Single binary copied from builder
```

**Runtime Configuration:**
```yaml
Ports:
  Host: 3000
  Container: 3000

Environment Variables:
  (Same as development backend)

Health Check:
  Command: curl --fail --silent http://127.0.0.1:3000/health || exit 1
  Interval: 30s
  Timeout: 5s
  Start Period: 20s
  Retries: 3

Restart Policy: unless-stopped

User: iviss (non-root)

Command: /app/iviss-backend
```

**Dependencies:**
- db (must be healthy)
- redis (must be healthy)

**Production Readiness:** ✅ Ready

**Image Size:** ~150-200 MB (optimized)

**Security:**
- ✅ Non-root user
- ✅ Minimal base image
- ✅ No build tools in final image
- ❌ No image scanning

**Activation:** Requires `--profile prod` flag

---

### 5. Frontend (Development)

**Image:** Custom (built from `frontend/Dockerfile`, target: `dev`)  
**Container Name:** `iviss-frontend`  
**Purpose:** React development server with hot-reload

**Base Image:** `node:20-alpine`

**Build Configuration:**
```dockerfile
Stage 1 (base): Copy package files
Stage 2 (deps): npm ci --legacy-peer-deps
Stage 3 (dev): Copy source + node_modules
```

**Runtime Configuration:**
```yaml
Ports:
  Host: 8080
  Container: 8080

Environment Variables:
  NODE_ENV: development
  VITE_API_URL: http://localhost:3000 (default)
  BACKEND_OPENAPI_URL: http://iviss-backend:3000/api-doc/openapi.json

Volumes:
  - ./frontend:/app (full source mount)
  - /app/node_modules (anonymous volume - prevents overwrite)
  - ./scripts:/scripts

Health Check:
  Command: wget --no-verbose --tries=1 --spider http://localhost:8080
  Interval: 10s
  Timeout: 5s
  Retries: 3

Restart Policy: unless-stopped

Command: npm run dev -- --host
```

**Dependencies:**
- backend (must be healthy)

**Hot Reload:** ✅ Enabled (Vite HMR)

**Production Readiness:** ❌ Development only

**Image Size:** ~500 MB (includes node_modules)

---

### 6. Frontend (Production)

**Image:** Custom (built from `frontend/Dockerfile`, target: `prod`)  
**Container Name:** Not named (profile-based)  
**Purpose:** Static assets served by Nginx

**Base Image:** `nginx:alpine`

**Build Strategy:**
```dockerfile
Stage 1 (base): Copy package files
Stage 2 (deps): Install dependencies
Stage 3 (builder): Build production assets
  - npm run codegen (generate API client)
  - npm run build (Vite production build)
Stage 4 (prod): Nginx runtime
  - Copy dist/ from builder
  - Copy i18n locales
  - Copy nginx.conf
```

**Runtime Configuration:**
```yaml
Ports:
  Host: 8080
  Container: 80

Nginx Configuration:
  - Gzip compression enabled
  - SPA routing (try_files fallback to index.html)
  - Static asset caching (30 days)
  - Error pages configured

Health Check:
  Command: wget --no-verbose --tries=1 --spider http://localhost
  Interval: 10s
  Timeout: 5s
  Retries: 3

Restart Policy: unless-stopped

Command: nginx -g "daemon off;"
```

**Dependencies:**
- backend-prod (must be healthy)

**Production Readiness:** ✅ Ready

**Image Size:** ~50-80 MB (static assets + Nginx)

**Security:**
- ✅ Minimal base image (alpine)
- ✅ No build tools in final image
- ❌ Nginx runs as root (default)
- ❌ No security headers configured

**Activation:** Requires `--profile prod` flag

---

### 7. Adminer (Database Admin)

**Image:** `adminer:latest`  
**Container Name:** `iviss-adminer`  
**Purpose:** Web-based database management UI

**Runtime Configuration:**
```yaml
Ports:
  Host: 8081
  Container: 8080

Restart Policy: unless-stopped

Logging:
  Driver: json-file
  Max Size: 10m
  Max Files: 3
```

**Dependencies:**
- db

**Production Readiness:** ❌ Development/debugging only

**Security Warning:** Should NOT be deployed to production (no authentication beyond database credentials)

---

### 8. Metrics Server

**Image:** Custom (built from `frontend/Dockerfile.metrics`)  
**Container Name:** `iviss-metrics-server`  
**Purpose:** Collect frontend metrics and expose Prometheus endpoint

**Base Image:** `node:20-alpine`

**Build Configuration:**
```dockerfile
- Install production dependencies only
- Copy metrics-server.js
- Expose port 9091
```

**Runtime Configuration:**
```yaml
Ports:
  Host: 9091
  Container: 9091

Environment Variables:
  NODE_ENV: production
  METRICS_PORT: 9091

Health Check:
  Command: node -e "fetch('http://localhost:9091/health').then(r => r.ok ? process.exit(0) : process.exit(1))"
  Interval: 30s
  Timeout: 10s
  Retries: 3

Restart Policy: unless-stopped

Logging:
  Driver: json-file
  Max Size: 10m
  Max Files: 3
```

**Dependencies:** None (standalone)

**Production Readiness:** ✅ Ready

**Image Size:** ~100 MB

**Endpoints:**
- POST /api/metrics - Receive metrics from browser
- GET /metrics - Prometheus scrape endpoint
- GET /health - Health check

---

## Network Architecture

### Docker Network

**Network Name:** `iviss-network`  
**Driver:** bridge  
**Subnet:** Auto-assigned by Docker  
**Gateway:** Auto-assigned by Docker

### Service Communication

```
External (Host)
    │
    ├─ :8080 → frontend (dev) or frontend-prod
    ├─ :3000 → backend (dev) or backend-prod
    ├─ :5435 → db
    ├─ :6380 → redis
    ├─ :8081 → adminer
    └─ :9091 → metrics

Internal (Container Network)
    │
    frontend → backend:3000 (API calls)
    backend → db:5432 (database queries)
    backend → redis:6379 (cache/sessions)
    adminer → db:5432 (database management)
    metrics → (standalone, scraped by external Prometheus)
```

### DNS Resolution

All services can resolve each other by container name:
- `iviss-db` → PostgreSQL
- `iviss-redis` → Redis
- `iviss-backend` → Backend API
- `iviss-frontend` → Frontend dev server
- `iviss-adminer` → Adminer
- `iviss-metrics-server` → Metrics server

### External DNS

Backend container configured with external DNS servers:
- 1.1.1.1 (Cloudflare)
- 8.8.8.8 (Google)

Purpose: Resolve external APIs (Twilio, future integrations)

---

## Volume Management

### Persistent Volumes

| Volume Name | Purpose | Size (Approx) | Backup Required |
|-------------|---------|---------------|-----------------|
| postgres_data | Database files | 500 MB - 10 GB | ✅ Yes |
| redis_data | Redis persistence | 10-100 MB | ⚠️ Optional |
| cargo_cache | Rust dependencies | 1-2 GB | ❌ No |
| target_cache | Rust build artifacts | 2-5 GB | ❌ No |

### Volume Lifecycle

**Creation:** Automatic on first `docker compose up`  
**Persistence:** Survives `docker compose down`  
**Deletion:** Only with `docker compose down -v` flag

### Backup Strategy (NOT IMPLEMENTED)

**Critical Gap:** No automated backup for postgres_data volume

**Recommended Backup Strategy:**
1. Daily automated backups using pg_dump
2. Backup retention: 30 days
3. Off-site backup storage
4. Regular restore testing

---

## Health Checks

### Health Check Summary

| Service | Endpoint | Interval | Timeout | Retries | Start Period |
|---------|----------|----------|---------|---------|--------------|
| db | pg_isready | 10s | 5s | 5 | 30s |
| redis | redis-cli ping | 10s | 5s | 5 | 10s |
| backend (dev) | GET /api/v1/health | 10s | 5s | 5 | 180s |
| backend-prod | GET /health | 30s | 5s | 3 | 20s |
| frontend (dev) | GET / | 10s | 5s | 3 | - |
| frontend-prod | GET / | 10s | 5s | 3 | - |
| metrics | GET /health | 30s | 10s | 3 | - |

### Health Check Behavior

**Healthy:** Container is ready to receive traffic  
**Unhealthy:** Container is running but not responding correctly  
**Starting:** Within start period, failures don't count as unhealthy

**Docker Compose Behavior:**
- `depends_on` with `condition: service_healthy` waits for health check
- Unhealthy containers are NOT automatically restarted
- Health status visible in `docker compose ps`

---

## Startup Order

### Dependency Graph

```
Level 1 (No dependencies):
  ├─ db
  ├─ redis
  └─ metrics

Level 2 (Depends on Level 1):
  ├─ backend (depends on: db, redis)
  ├─ backend-prod (depends on: db, redis)
  └─ adminer (depends on: db)

Level 3 (Depends on Level 2):
  ├─ frontend (depends on: backend)
  └─ frontend-prod (depends on: backend-prod)
```

### Startup Sequence

1. Docker Compose starts Level 1 services in parallel
2. Waits for db and redis to become healthy
3. Starts backend (waits for health check)
4. Starts frontend (waits for backend health check)
5. Starts adminer (no health check wait)
6. Starts metrics (independent)

**Total Startup Time:**
- Development: ~3-5 minutes (backend compilation)
- Production: ~30-60 seconds

---

## Resource Profiles

### Current Resource Allocation

**Configured Limits:**
- Redis: 0.25 CPU, 256 MB RAM

**No Limits Configured:**
- PostgreSQL
- Backend
- Frontend
- Adminer
- Metrics

**Risk:** Services can consume all available host resources

### Recommended Production Limits

| Service | CPU Limit | Memory Limit | Notes |
|---------|-----------|--------------|-------|
| db | 2 cores | 2 GB | Adjust based on load |
| redis | 0.5 cores | 512 MB | Current limit too low |
| backend-prod | 1 core | 1 GB | Per instance |
| frontend-prod | 0.25 cores | 256 MB | Static assets only |
| metrics | 0.25 cores | 256 MB | Lightweight |

### Scaling Considerations

**Current Architecture:** Single instance per service (no scaling)

**Horizontal Scaling Requirements:**
- Load balancer (not present)
- Session affinity or shared session storage (Redis already used)
- Database connection pooling (implemented in backend)
- Shared file storage (not needed - stateless services)

**Vertical Scaling:**
- Increase container resource limits
- Upgrade host machine
- No code changes required

---

## Container Security

### Current Security Posture

✅ **Implemented:**
- Non-root user in backend-prod container
- Minimal base images (alpine, slim)
- Read-only source mounts in development
- Health checks for all services
- Logging configured

❌ **Missing:**
- Container image scanning
- Security context constraints
- AppArmor/SELinux profiles
- Secrets management (using environment variables)
- Network policies
- Resource quotas
- Pod security policies (N/A - not using Kubernetes)

### Security Recommendations

1. **Image Scanning:**
   - Implement Trivy or Snyk in CI/CD
   - Scan for CVEs before deployment
   - Block high-severity vulnerabilities

2. **Runtime Security:**
   - Run all containers as non-root
   - Use read-only root filesystems where possible
   - Drop unnecessary capabilities
   - Implement seccomp profiles

3. **Network Security:**
   - Implement network policies (requires orchestrator)
   - Use TLS for inter-service communication
   - Restrict egress traffic

4. **Secrets Management:**
   - Replace environment variables with Docker secrets
   - Use external secrets manager (Vault, AWS Secrets Manager)
   - Rotate secrets regularly

---

## Logging & Monitoring

### Logging Configuration

**All Services:**
```yaml
Logging:
  Driver: json-file
  Options:
    max-size: 10m
    max-file: 3
```

**Total Log Storage per Service:** 30 MB (3 files × 10 MB)

**Log Rotation:** Automatic when file reaches 10 MB

**Log Location:** `/var/lib/docker/containers/<container-id>/<container-id>-json.log`

### Log Aggregation (NOT IMPLEMENTED)

**Current State:** Logs only accessible via `docker compose logs`

**Recommended Solution:**
- ELK Stack (Elasticsearch, Logstash, Kibana)
- Loki + Grafana
- Cloud-native solutions (CloudWatch, Stackdriver)

### Monitoring (Partial Implementation)

**Implemented:**
- Frontend metrics (Prometheus + Grafana)
- Health checks

**Missing:**
- Backend application metrics
- Database performance metrics
- Redis metrics
- Container resource metrics
- Alerting

---

## Deployment Profiles

### Development Profile (Default)

**Command:** `docker compose up -d`

**Services Started:**
- db
- redis
- backend (dev mode with hot-reload)
- frontend (dev mode with Vite)
- adminer
- metrics

**Features:**
- Hot-reload enabled
- Source code mounted
- Debug logging
- Database admin UI

**Use Case:** Local development

---

### Production Profile

**Command:** `docker compose --profile prod up -d db redis backend-prod frontend-prod metrics`

**Services Started:**
- db
- redis
- backend-prod (optimized binary)
- frontend-prod (Nginx + static assets)
- metrics

**Features:**
- Optimized builds
- No source mounts
- Production logging
- Minimal images

**Use Case:** Local testing of production builds

**Note:** This is NOT actual production deployment, just production-like containers running locally.

---

## Container Build Process

### Backend Build

**Dockerfile:** `iviss-backend/Dockerfile`

**Build Stages:**
1. **base:** Install system dependencies
2. **development:** Add cargo-watch, warm cache
3. **builder:** Multi-stage build with caching
4. **production:** Minimal runtime image

**Build Command:**
```bash
docker build -t iviss-backend:dev --target development ./iviss-backend
docker build -t iviss-backend:prod --target production ./iviss-backend
```

**Build Time:**
- Development: ~10-15 minutes (first build)
- Development: ~30 seconds (cached)
- Production: ~15-20 minutes (first build)
- Production: ~2-3 minutes (cached)

**Build Optimizations:**
- Dependency caching (separate layer)
- Multi-stage builds
- Layer caching in CI/CD

---

### Frontend Build

**Dockerfile:** `frontend/Dockerfile`

**Build Stages:**
1. **base:** Copy package files
2. **deps:** Install dependencies
3. **dev:** Development server
4. **builder:** Production build
5. **prod:** Nginx runtime

**Build Command:**
```bash
docker build -t iviss-frontend:dev --target dev ./frontend
docker build -t iviss-frontend:prod --target prod ./frontend
```

**Build Time:**
- Development: ~5-10 minutes (first build)
- Development: ~10 seconds (cached)
- Production: ~5-10 minutes (first build)
- Production: ~1-2 minutes (cached)

---

## Troubleshooting Guide

### Common Issues

**Issue:** Backend container exits immediately
- **Cause:** Database not ready
- **Fix:** Check db health with `docker compose ps`

**Issue:** Frontend can't connect to backend
- **Cause:** VITE_API_URL misconfigured
- **Fix:** Check frontend/.env file

**Issue:** Database data lost after restart
- **Cause:** Volume deleted with `docker compose down -v`
- **Fix:** Use `docker compose down` without `-v` flag

**Issue:** Port already in use
- **Cause:** Another service using the port
- **Fix:** Stop conflicting service or change port in docker-compose.yml

**Issue:** Backend compilation fails in dev mode
- **Cause:** Rust dependencies changed
- **Fix:** Rebuild with `docker compose up --build backend`

### Debugging Commands

```bash
# View logs
docker compose logs -f backend
docker compose logs -f frontend

# Check service health
docker compose ps

# Inspect container
docker compose exec backend sh

# View resource usage
docker stats

# Restart single service
docker compose restart backend

# Rebuild single service
docker compose up -d --build backend

# View volumes
docker volume ls

# Inspect volume
docker volume inspect iviss_postgres_data
```

---

## Migration to Production Orchestration

### Current State: Docker Compose

**Limitations:**
- Single host only
- No automatic scaling
- No rolling updates
- No service discovery beyond DNS
- No load balancing
- Manual failover

### Recommended Migration Path

**Option 1: Kubernetes**
- Convert docker-compose.yml to Kubernetes manifests
- Use Helm charts for templating
- Implement Ingress for routing
- Use StatefulSets for databases
- Implement HorizontalPodAutoscaler

**Option 2: Docker Swarm**
- Minimal changes from Docker Compose
- Built-in load balancing
- Rolling updates
- Simpler than Kubernetes
- Limited ecosystem

**Option 3: Managed Container Services**
- AWS ECS/Fargate
- Azure Container Instances
- Google Cloud Run
- Minimal infrastructure management
- Cloud vendor lock-in

---

## Handover Checklist

### Immediate Actions

1. **Review Container Configuration**
   - [ ] Verify all health checks work
   - [ ] Test startup order
   - [ ] Validate resource limits

2. **Test Production Builds**
   - [ ] Build production images locally
   - [ ] Test with `--profile prod`
   - [ ] Verify functionality

3. **Document Custom Images**
   - [ ] Review Dockerfiles
   - [ ] Understand build stages
   - [ ] Document build process

4. **Plan Resource Allocation**
   - [ ] Determine production resource needs
   - [ ] Set appropriate limits
   - [ ] Plan for scaling

5. **Implement Security Measures**
   - [ ] Add image scanning
   - [ ] Review security contexts
   - [ ] Implement secrets management

---

## Future Enhancements

### Short Term
- Add resource limits to all services
- Implement container image scanning
- Add backend application metrics
- Configure security contexts

### Medium Term
- Migrate to Kubernetes or similar
- Implement horizontal pod autoscaling
- Add service mesh (Istio, Linkerd)
- Implement GitOps (ArgoCD, Flux)

### Long Term
- Multi-region deployment
- Advanced observability
- Chaos engineering
- Self-healing infrastructure

---

## References

- Docker Compose Documentation: https://docs.docker.com/compose/
- Docker Best Practices: https://docs.docker.com/develop/dev-best-practices/
- Multi-stage Builds: https://docs.docker.com/build/building/multi-stage/
- Health Checks: https://docs.docker.com/engine/reference/builder/#healthcheck

---

## Document Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | April 2026 | DevOps Analysis | Initial container architecture documentation |
