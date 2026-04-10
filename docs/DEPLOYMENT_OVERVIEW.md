# IVISS Deployment Overview

**Document Version:** 1.0  
**Last Updated:** April 2026  
**Target Audience:** DevOps/Deployment Team

---

## Executive Summary

IVISS is currently in **early-stage development** with a **Docker-based local development setup**. The project has basic CI/CD automation for testing and image building, but **no production deployment infrastructure exists yet**. All deployment activities are currently manual and local-only.

### Current Deployment Maturity: **Level 1 - Local Development**

- ✅ Docker Compose for local development
- ✅ CI/CD pipelines for testing and image building
- ✅ Container images published to GitHub Container Registry (GHCR)
- ❌ No production hosting infrastructure
- ❌ No Infrastructure as Code (IaC)
- ❌ No automated deployment to any environment
- ❌ No staging/production environments configured
- ❌ No secrets management solution
- ❌ No rollback mechanisms
- ❌ No load balancing or high availability

---

## What Exists Today

### 1. Local Development Environment
- Docker Compose orchestration with 7 services
- Hot-reload enabled for backend (Rust) and frontend (React)
- PostgreSQL 15 + Redis 7 for data storage
- Adminer for database management
- Prometheus + Grafana for frontend metrics (optional)

### 2. CI/CD Automation
- GitHub Actions workflows for:
  - Backend testing, linting, security audits
  - Frontend testing, linting, type checking
  - Docker image building and publishing to GHCR
- Automated on push to `main` and `dev` branches
- Pull request validation

### 3. Container Images
- Production-ready Docker images built via multi-stage builds
- Published to GitHub Container Registry (ghcr.io)
- Tagged with branch name, commit SHA, and `latest`

### 4. Configuration Management
- Environment variables via `.env` files
- Separate configs for development and production builds
- No secrets management solution (secrets in plain text `.env` files)

---

## What Does NOT Exist

### Critical Gaps

1. **No Production Infrastructure**
   - No servers, VMs, or cloud resources provisioned
   - No hosting platform selected
   - No domain or DNS configuration

2. **No Deployment Automation**
   - Images are built but never deployed anywhere
   - No deployment scripts or tools
   - No environment promotion workflow

3. **No Infrastructure as Code**
   - No Terraform, Ansible, Kubernetes manifests, or Helm charts
   - All infrastructure setup would be manual

4. **No Secrets Management**
   - JWT keys, database passwords, API tokens stored in `.env` files
   - No HashiCorp Vault, AWS Secrets Manager, or similar

5. **No Monitoring in Production**
   - Prometheus/Grafana only configured for local development
   - No alerting, logging aggregation, or APM

6. **No Backup/Disaster Recovery**
   - No database backup strategy
   - No disaster recovery plan

7. **No High Availability**
   - Single instance architecture
   - No load balancing
   - No failover mechanisms

---

## Current Deployment Flow (Local Only)

```
Developer Workstation
         │
         ├─ Edit code
         ├─ git commit
         ├─ git push
         │
         ▼
    GitHub Repository
         │
         ├─ Trigger CI/CD (GitHub Actions)
         │   ├─ Run tests
         │   ├─ Build Docker images
         │   └─ Push to GHCR
         │
         ▼
GitHub Container Registry (GHCR)
         │
         │  [MANUAL STEP - No automation beyond this point]
         │
         ▼
    Developer runs:
    docker compose up -d
         │
         ▼
    Local Docker Engine
    (Services running on localhost)
```

**Key Point:** There is no automated deployment to any environment beyond the developer's local machine.

---

## Technology Stack

### Application Services
- **Backend:** Rust 1.89 + Axum web framework
- **Frontend:** React 18 + TypeScript + Vite
- **Database:** PostgreSQL 15
- **Cache:** Redis 7
- **Web Server:** Nginx (for production frontend)

### Development Tools
- **Container Runtime:** Docker + Docker Compose
- **CI/CD:** GitHub Actions
- **Container Registry:** GitHub Container Registry (GHCR)
- **Monitoring (Local):** Prometheus + Grafana
- **Database Admin:** Adminer

### Missing Production Tools
- Load Balancer: None
- Reverse Proxy: None (beyond Nginx in frontend container)
- Secrets Manager: None
- Log Aggregation: None
- APM/Tracing: None
- Backup Solution: None

---

## Service Architecture

### Container Services (7 total)

| Service | Image | Purpose | Ports | Status |
|---------|-------|---------|-------|--------|
| `db` | postgres:15-alpine | Primary database | 5435→5432 | ✅ Running |
| `redis` | redis:7-alpine | Cache & OTP storage | 6380→6379 | ✅ Running |
| `backend` | Custom (dev target) | API server (dev mode) | 3000→3000 | ✅ Running |
| `backend-prod` | Custom (prod target) | API server (prod mode) | 3000→3000 | ⚠️ Opt-in profile |
| `frontend` | Custom (dev target) | React app (dev mode) | 8080→8080 | ✅ Running |
| `frontend-prod` | Custom (prod target) | React app (prod mode) | 8080→80 | ⚠️ Opt-in profile |
| `adminer` | adminer:latest | DB admin UI | 8081→8080 | ✅ Running |
| `metrics` | Custom | Metrics collector | 9091→9091 | ✅ Running |

**Note:** `backend-prod` and `frontend-prod` are only started with `--profile prod` flag.

### Service Dependencies

```
frontend → backend → db
                  → redis

metrics → (standalone, scraped by Prometheus)
adminer → db
```

### Network Architecture
- All services on single Docker bridge network: `iviss-network`
- No external network access configured
- No TLS/SSL termination
- No reverse proxy

---

## Resource Requirements

### Current Resource Profile (Local Development)

| Service | CPU Limit | Memory Limit | Storage |
|---------|-----------|--------------|---------|
| PostgreSQL | None | None | ~500MB (volume) |
| Redis | 0.25 cores | 256MB | ~10MB (volume) |
| Backend | None | None | ~2GB (build cache) |
| Frontend | None | None | ~500MB (node_modules) |
| Adminer | None | None | Minimal |
| Metrics | None | None | Minimal |

**Total Estimated:** ~2-4GB RAM, 2-4 CPU cores for comfortable local development

### Production Resource Estimates (Not Yet Defined)

The deployment team will need to determine:
- Expected concurrent users
- Database size projections
- API request volume
- Appropriate instance sizing

---

## Data Persistence

### Persistent Volumes

| Volume | Purpose | Backup Strategy |
|--------|---------|-----------------|
| `postgres_data` | Database files | ❌ None |
| `redis_data` | Redis persistence | ❌ None |
| `cargo_cache` | Rust build cache | Not needed |
| `target_cache` | Rust compilation artifacts | Not needed |

**Critical Gap:** No backup strategy exists for production data.

---

## Configuration Management

### Environment Variables

Configuration is managed through `.env` files:

**Root `.env`** (Docker Compose level):
- Database credentials
- JWT keys (RSA private/public key pair)
- Twilio SMS credentials
- Admin bootstrap credentials
- Shift hours configuration

**Backend `.env`** (iviss-backend/.env):
- Database URLs (internal + external)
- Redis URL
- JWT configuration
- Server host/port
- Logging level
- SMS provider settings

**Frontend `.env`** (frontend/.env):
- API URL
- Metrics configuration

### Secrets Handling (Current State)

⚠️ **CRITICAL SECURITY GAP:**
- All secrets stored in plain text `.env` files
- `.env.example` files committed to git (with placeholder values)
- Actual `.env` files in `.gitignore` but no secure distribution method
- JWT private keys stored as environment variables
- No encryption at rest
- No secret rotation mechanism

**Required for Production:**
- Implement proper secrets management (Vault, AWS Secrets Manager, etc.)
- Rotate all development secrets before production
- Generate production-specific JWT keys
- Secure Twilio credentials
- Implement secret rotation policy

---

## Database Management

### Migration Strategy

- **Tool:** SQLx migrations (Rust)
- **Location:** `iviss-backend/migrations/`
- **Execution:** Automatic on backend startup
- **Current Migrations:** 22 migration files

### Migration Process

1. Backend container starts
2. SQLx checks `_sqlx_migrations` table
3. Applies pending migrations in order
4. Application starts

**Gaps:**
- No rollback mechanism for failed migrations
- No migration testing in staging environment
- No database backup before migrations
- No migration approval process

### Seed Data

- **Location:** `iviss-backend/seeds/seed_data.sql`
- **Purpose:** Development test data
- **Execution:** Manual (not automated)

---

## Monitoring & Observability

### What Exists (Local Development Only)

**Frontend Metrics:**
- Prometheus scraping metrics server (port 9091)
- Grafana dashboards (port 3001)
- Metrics collected:
  - Page load duration
  - Web Vitals (LCP, FID, CLS)
  - Active sessions
  - JavaScript errors
  - Route navigations

**Health Checks:**
- Backend: `GET /api/v1/health`
- Frontend: HTTP 200 on root path
- Database: `pg_isready` check
- Redis: `redis-cli ping`

### What Does NOT Exist

- ❌ Backend application metrics (no instrumentation)
- ❌ Database performance monitoring
- ❌ Log aggregation (ELK, Loki, etc.)
- ❌ Distributed tracing (Jaeger, Zipkin)
- ❌ Alerting rules
- ❌ On-call rotation
- ❌ Incident response procedures
- ❌ SLA/SLO definitions

---

## Security Posture

### Current Security Measures

✅ **Implemented:**
- Gitleaks scanning in CI/CD (secret detection)
- Cargo audit for Rust dependency vulnerabilities
- JWT RS256 token authentication
- Argon2 password hashing
- CORS configuration
- Rate limiting (in application code)

❌ **Missing:**
- TLS/SSL certificates
- Network segmentation
- Firewall rules
- DDoS protection
- WAF (Web Application Firewall)
- Intrusion detection
- Security scanning in production
- Penetration testing
- Compliance certifications (if required)

---

## Handover Checklist for Deployment Team

### Immediate Actions Required

1. **Select Hosting Platform**
   - [ ] Evaluate hosting options
   - [ ] Determine deployment model
   - [ ] Estimate costs and resource needs

2. **Design Production Architecture**
   - [ ] Define environment strategy
   - [ ] Plan network topology
   - [ ] Design high availability approach
   - [ ] Plan disaster recovery

3. **Implement Secrets Management**
   - [ ] Choose secrets management approach
   - [ ] Migrate secrets from plain text files
   - [ ] Generate production-specific keys
   - [ ] Implement secret rotation policy

4. **Set Up Infrastructure as Code**
   - [ ] Choose IaC tooling
   - [ ] Define infrastructure modules
   - [ ] Implement environment provisioning
   - [ ] Set up state management

5. **Configure CI/CD for Deployment**
   - [ ] Add deployment stages to pipelines
   - [ ] Implement environment promotion workflow
   - [ ] Add approval gates for production
   - [ ] Configure rollback mechanisms

6. **Implement Monitoring**
   - [ ] Set up production monitoring stack
   - [ ] Configure alerting rules
   - [ ] Implement log aggregation
   - [ ] Set up on-call rotation

7. **Plan Data Management**
   - [ ] Implement database backup strategy
   - [ ] Test restore procedures
   - [ ] Plan data retention policies
   - [ ] Configure database replication if needed

8. **Security Hardening**
   - [ ] Obtain SSL/TLS certificates
   - [ ] Configure firewall rules
   - [ ] Implement network segmentation
   - [ ] Set up security scanning
   - [ ] Conduct security audit

### Questions for Stakeholders

1. What is the target go-live date?
2. What is the expected user load?
3. What are the uptime requirements (SLA)?
4. What is the budget for infrastructure?
5. Are there compliance requirements (GDPR, HIPAA, etc.)?
6. What is the disaster recovery RTO/RPO?
7. Is multi-region deployment required?
8. What are the data residency requirements?

---

## Next Steps

### Phase 1: Foundation (Weeks 1-2)
- Select hosting platform
- Set up development/staging/production environments
- Implement secrets management
- Create basic IaC for infrastructure provisioning

### Phase 2: Deployment Pipeline (Weeks 3-4)
- Extend CI/CD to include deployment stages
- Implement automated deployment to staging
- Add manual approval for production deployment
- Test rollback procedures

### Phase 3: Production Readiness (Weeks 5-6)
- Set up production monitoring and alerting
- Implement backup and disaster recovery
- Security hardening and audit
- Load testing and performance tuning

### Phase 4: Go-Live (Week 7+)
- Production deployment
- Post-deployment monitoring
- Incident response readiness
- Documentation handover to operations team

---

## Contact Information

**Development Team Lead:** [To be filled]  
**DevOps Team Lead:** [To be filled]  
**Project Manager:** [To be filled]

---

## Document Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | April 2026 | DevOps Analysis | Initial deployment assessment |
