# IVISS Deployment Documentation Index

**Last Updated:** April 2026

---

## Quick Start for Deployment Team

**New to the project? Start here:**

1. **[DEPLOYMENT_OVERVIEW.md](DEPLOYMENT_OVERVIEW.md)** ⭐ START HERE
   - Current deployment maturity assessment
   - What exists vs. what's missing
   - Technology stack overview
   - Handover checklist

2. **[DEPLOYMENT_INDEX.md](DEPLOYMENT_INDEX.md)** (this document)
   - Navigation hub for all deployment documentation

---

## Technical Deep Dives

### CI/CD and Automation
**[CICD_PIPELINES.md](CICD_PIPELINES.md)**
- GitHub Actions workflows explained
- Build and test automation
- Container image publishing
- Missing deployment stages
- Failure handling and rollback strategies

### Container Architecture
**[CONTAINER_ARCHITECTURE.md](CONTAINER_ARCHITECTURE.md)**
- All 8 container services detailed
- Docker Compose configuration
- Network architecture
- Volume management
- Health checks and startup order
- Resource requirements
- Security posture

### Infrastructure and Hosting
**[INFRASTRUCTURE_AND_HOSTING.md](INFRASTRUCTURE_AND_HOSTING.md)**
- Current state: NO infrastructure exists
- Hosting platform options (AWS, Azure, GCP, VPS)
- Network architecture requirements
- DNS and SSL/TLS setup
- Database and cache infrastructure
- Monitoring and backup strategies
- Cost estimates
- Security infrastructure

---

## Application Documentation

### System Overview
**[overview.md](overview.md)**
- What IVISS does
- Who uses it
- System components
- Current implementation status

### Technical Architecture
**[architecture_spec.md](architecture_spec.md)**
- Backend structure (Rust + Axum)
- Frontend structure (React + TypeScript)
- User roles and authentication
- Key technologies

### Component Guide
**[components.md](components.md)**
- Backend handlers and services
- Frontend pages and components
- How components work together
- Common workflows

### Database Schema
**[schema.md](schema.md)** - Detailed with business context  
**[schema_simple.md](schema_simple.md)** - Simplified overview

### Data Models
**[data.md](data.md)**
- TypeScript interfaces
- API data structures
- Status values and constants

---

## Operational Documentation

### Docker Setup
**[docker_setup.md](docker_setup.md)**
- Local development setup
- Production-like local testing
- Troubleshooting

### Monitoring
**[monitoring.md](monitoring.md)**
- Prometheus + Grafana setup (local only)
- Frontend metrics collection
- Dashboard configuration

### Authentication Flows
**[auth_tokens.md](auth_tokens.md)** - JWT token implementation  
**[daily_opertational_login_flow.md](daily_opertational_login_flow.md)** - Agent OTP login  
**[auto_refresh_signature.md](auto_refresh_signature.md)** - Token refresh mechanism

---

## Documentation by Role

### For DevOps Lead
**Must Read:**
1. DEPLOYMENT_OVERVIEW.md
2. INFRASTRUCTURE_AND_HOSTING.md
3. CICD_PIPELINES.md

**Reference:**
- CONTAINER_ARCHITECTURE.md
- docker_setup.md

### For DevOps Engineers
**Must Read:**
1. DEPLOYMENT_OVERVIEW.md
2. CONTAINER_ARCHITECTURE.md
3. CICD_PIPELINES.md
4. docker_setup.md

**Reference:**
- INFRASTRUCTURE_AND_HOSTING.md
- architecture_spec.md
- monitoring.md

### For Database Administrator
**Must Read:**
1. schema.md or schema_simple.md
2. CONTAINER_ARCHITECTURE.md (database section)
3. INFRASTRUCTURE_AND_HOSTING.md (database section)

**Reference:**
- data.md
- DEPLOYMENT_OVERVIEW.md

### For Security Engineer
**Must Read:**
1. DEPLOYMENT_OVERVIEW.md (security gaps)
2. INFRASTRUCTURE_AND_HOSTING.md (security section)
3. CONTAINER_ARCHITECTURE.md (security section)
4. auth_tokens.md

**Reference:**
- CICD_PIPELINES.md (security scanning)
- architecture_spec.md

### For SRE/Operations
**Must Read:**
1. DEPLOYMENT_OVERVIEW.md
2. CONTAINER_ARCHITECTURE.md
3. monitoring.md
4. INFRASTRUCTURE_AND_HOSTING.md (monitoring section)

**Reference:**
- CICD_PIPELINES.md
- docker_setup.md

---

## Documentation by Phase

### Phase 1: Understanding (Week 1)
**Goal:** Understand current state

**Read:**
1. DEPLOYMENT_HANDOVER.md
2. DEPLOYMENT_OVERVIEW.md
3. overview.md
4. architecture_spec.md

**Action:** Make key decisions (hosting platform, etc.)

---

### Phase 2: Planning (Week 1-2)
**Goal:** Design production infrastructure

**Read:**
1. INFRASTRUCTURE_AND_HOSTING.md
2. CONTAINER_ARCHITECTURE.md
3. schema.md

**Action:** Create infrastructure design documents

---

### Phase 3: Implementation (Week 2-4)
**Goal:** Build infrastructure and automation

**Read:**
1. CICD_PIPELINES.md
2. CONTAINER_ARCHITECTURE.md
3. docker_setup.md

**Action:** Write IaC, extend CI/CD, provision infrastructure

---

### Phase 4: Deployment (Week 5-6)
**Goal:** Deploy to production

**Read:**
1. All deployment docs (review)
2. monitoring.md
3. auth_tokens.md (for troubleshooting)

**Action:** Deploy, monitor, validate

---

## Critical Information Summary

### Current State
- ✅ Application code complete and working
- ✅ Docker containers for all services
- ✅ CI/CD for testing and image building
- ❌ NO production infrastructure
- ❌ NO deployment automation
- ❌ NO secrets management
- ❌ NO production monitoring

### Technology Stack
- **Backend:** Rust 1.89 + Axum
- **Frontend:** React 18 + TypeScript + Vite
- **Database:** PostgreSQL 15
- **Cache:** Redis 7
- **Containers:** Docker + Docker Compose
- **CI/CD:** GitHub Actions
- **Registry:** GitHub Container Registry

### Services (8 total)
1. PostgreSQL database
2. Redis cache
3. Backend API (dev + prod variants)
4. Frontend SPA (dev + prod variants)
5. Adminer (dev only)
6. Metrics server

### Ports
- Frontend: 8080
- Backend: 3000
- Database: 5435
- Redis: 6380
- Adminer: 8081
- Metrics: 9091

### Critical Gaps
1. No production infrastructure
2. No secrets management
3. No deployment automation
4. No backup strategy
5. No production monitoring
6. No disaster recovery plan
7. No high availability
8. No load balancing

---

## Quick Links

### GitHub
- **Repository:** https://github.com/<owner>/iviss
- **Actions:** https://github.com/<owner>/iviss/actions
- **Packages:** https://github.com/<owner>/iviss/packages

### Local Development
- **Frontend:** http://localhost:8080
- **Backend:** http://localhost:3000
- **API Docs:** http://localhost:3000/docs
- **Database Admin:** http://localhost:8081
- **Metrics:** http://localhost:9091

---

## Document Status

| Document | Status | Last Updated | Completeness |
|----------|--------|--------------|--------------|
| DEPLOYMENT_OVERVIEW.md | ✅ Complete | April 2026 | 100% |
| CICD_PIPELINES.md | ✅ Complete | April 2026 | 100% |
| CONTAINER_ARCHITECTURE.md | ✅ Complete | April 2026 | 100% |
| INFRASTRUCTURE_AND_HOSTING.md | ✅ Complete | April 2026 | 100% |
| overview.md | ✅ Complete | April 2026 | 100% |
| architecture_spec.md | ✅ Complete | April 2026 | 100% |
| components.md | ✅ Complete | April 2026 | 100% |
| schema.md | ✅ Complete | Earlier | 100% |
| schema_simple.md | ✅ Complete | April 2026 | 100% |
| data.md | ✅ Complete | Earlier | 100% |
| docker_setup.md | ✅ Complete | Earlier | 100% |
| monitoring.md | ✅ Complete | Earlier | 100% |
| auth_tokens.md | ✅ Complete | Earlier | 100% |

---

## Feedback and Updates

This documentation was created through comprehensive analysis of the IVISS codebase as of April 2026.

**To update this documentation:**
1. Edit the relevant markdown file
2. Update the "Last Updated" date
3. Update the Document Revision History section
4. Commit changes to git

**For questions or clarifications:**
- Contact the development team
- Review the code directly
- Check GitHub Issues for known problems

---

## Document Conventions

### Status Indicators
- ✅ Implemented/Complete
- ⚠️ Partially Implemented
- ❌ Not Implemented/Missing

### Priority Levels
- 🔴 Critical - Must address immediately
- 🟡 High - Address in first month
- 🟢 Medium - Address in first quarter
- ⚪ Low - Nice to have

### Document Types
- **Overview** - High-level summary
- **Technical** - Detailed technical information
- **Operational** - Day-to-day operations
- **Reference** - Quick lookup information

---

## Glossary

**Common terms used in deployment documentation:**

- **CI/CD:** Continuous Integration/Continuous Deployment
- **IaC:** Infrastructure as Code (Terraform, etc.)
- **GHCR:** GitHub Container Registry
- **SLA:** Service Level Agreement
- **RTO:** Recovery Time Objective
- **RPO:** Recovery Point Objective
- **VPC:** Virtual Private Cloud
- **ALB:** Application Load Balancer
- **RDS:** Relational Database Service
- **APM:** Application Performance Monitoring
- **WAF:** Web Application Firewall
- **CDN:** Content Delivery Network

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | April 2026 | Initial deployment documentation package |

---

**For immediate assistance, start with [DEPLOYMENT_OVERVIEW.md](DEPLOYMENT_OVERVIEW.md)**
