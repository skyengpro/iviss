# IVISS Deployment - Current State (Updated)

**Document Version:** 2.0  
**Last Updated:** April 17, 2026  
**Status:** ✅ PRODUCTION INFRASTRUCTURE IMPLEMENTED

---

## Executive Summary

**MAJOR UPDATE:** IVISS now has a **complete production deployment infrastructure** implemented using Terraform + Ansible + GitHub Actions. The system can be automatically deployed to AWS Lightsail with a single workflow trigger.

### Deployment Maturity: **Level 3 - Automated Production Deployment**

**What Changed Since Last Documentation:**

- ✅ **Production infrastructure implemented** (AWS Lightsail)
- ✅ **Infrastructure as Code** (Terraform)
- ✅ **Configuration Management** (Ansible)
- ✅ **Automated deployment pipeline** (GitHub Actions → Terraform → Ansible)
- ✅ **Secrets management** (GitHub Secrets → Ansible templates → EC2 .env)
- ✅ **SSL/TLS certificates** (Let's Encrypt via Certbot)
- ✅ **Reverse proxy** (Nginx)
- ✅ **Firewall configuration** (UFW)

---

## Current Architecture

### Deployment Flow

```
Developer Push to main
         │
         ▼
GitHub Actions (deploy-aws.yml)
         │
         ├─ Checkout code
         ├─ Setup Terraform
         ├─ Setup Ansible
         ├─ Configure AWS credentials
         │
         ▼
Terraform (infra/terraform/)
         │
         ├─ Provision AWS Lightsail instance
         ├─ Create static IP
         ├─ Configure firewall (ports 22, 80, 443)
         ├─ Generate SSH key pair
         │
         ▼
Ansible (infra/ansible/)
         │
         ├─ Install Docker, Nginx, UFW, Certbot
         ├─ Configure firewall rules
         ├─ Setup SSL certificate (Let's Encrypt)
         ├─ Copy docker-compose.yml
         ├─ Generate .env file with secrets
         ├─ Pull Docker images from GHCR
         ├─ Start containers with docker compose
         │
         ▼
AWS Lightsail Instance (Running)
         │
         ├─ PostgreSQL (container)
         ├─ Backend API (container)
         ├─ Frontend SPA (container)
         ├─ Nginx (reverse proxy)
         └─ SSL/TLS (Let's Encrypt)
```

---

## Infrastructure Details

### AWS Lightsail Instance

**Specifications:**

- **Instance Type:** small_3_0
- **vCPUs:** 2
- **RAM:** 2 GB
- **Storage:** 60 GB SSD
- **OS:** Ubuntu 22.04 LTS
- **Region:** eu-west-1 (Ireland)
- **Static IP:** Yes (persistent across restarts)

**Firewall Rules:**

- Port 22 (SSH)
- Port 80 (HTTP)
- Port 443 (HTTPS)

**Cost:** ~$12-15/month (AWS Lightsail pricing)

### Terraform State Management

**Backend:** AWS S3 + DynamoDB

- **Bucket:** `iviss-terraform-state-577638362880`
- **Key:** `production/terraform.tfstate`
- **Region:** eu-central-1
- **Lock Table:** `iviss-terraform-lock`
- **Encryption:** Enabled

**Purpose:** Centralized state management for team collaboration and state locking

---

## Deployment Process

### Automatic Deployment (Main Branch)

**Trigger:** Push to `main` branch (after Docker workflow completes)

**Workflow:** `.github/workflows/deploy-aws.yml`

**Steps:**

1. Docker images built and pushed to GHCR
2. Deploy workflow triggered automatically
3. Terraform provisions/updates infrastructure
4. Ansible configures server and deploys application
5. Services start automatically

**Duration:** ~10-15 minutes (first deploy), ~5-8 minutes (updates)

### Manual Deployment

**Trigger:** Push to `dep/iviss-aws-lightsail` branch

**Use Case:** Testing deployment without affecting main branch

### Deployment Script

**Location:** `infra/scripts/deploy.sh`

**Features:**

- Loads environment variables from `.env`
- Runs Terraform to provision infrastructure
- Generates Ansible inventory dynamically
- Handles multi-line PEM keys safely (Python script)
- Passes secrets to Ansible via JSON file
- Cleans up sensitive files after deployment

**Usage:**

```bash
./infra/scripts/deploy.sh "your-domain.com" "admin@example.com"
```

---

## Secrets Management

### Flow: GitHub Secrets → EC2 .env File

**1. GitHub Repository Secrets**

Stored in: `Settings → Secrets and variables → Actions`

Required secrets:

- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `REGISTRY_USERNAME` (GitHub username)
- `REGISTRY_TOKEN` (GitHub PAT)
- `JWT_PRIVATE_KEY_PEM`
- `JWT_PUBLIC_KEY_PEM`
- `ACTIVATION_CODE_PEPPER`
- `ADMIN_BOOTSTRAP_EMAIL`
- `ADMIN_BOOTSTRAP_PASSWORD`
- `ADMIN_BOOTSTRAP_PHONE`
- `ADMIN_BOOTSTRAP_USERNAME`
- `POSTGRES_USER`
- `POSTGRES_PASSWORD`
- `POSTGRES_DB`
- `TWILIO_ACCOUNT_SID`
- `TWILIO_AUTH_TOKEN`
- `TWILIO_FROM_NUMBER`
- `DOMAIN_NAME`
- `CERTBOT_EMAIL`

**2. GitHub Actions Workflow**

Passes secrets as environment variables to `deploy.sh`

**3. Deploy Script**

Creates JSON file with secrets:

```bash
python3 -c "
import json, os
vars = {
    'db_password': os.environ.get('POSTGRES_PASSWORD'),
    'jwt_private_key_pem': get_pem('JWT_PRIVATE_KEY_PEM'),
    # ... etc
}
with open('.deploy-vars.json', 'w') as f:
    json.dump(vars, f)
"
```

**4. Ansible**

Receives secrets via `--extra-vars "@.deploy-vars.json"`

**5. Jinja2 Templates**

Generate `.env` file on EC2:

- `dotenv.j2` → `/opt/iviss/.env`

**6. Docker Compose**

Reads `/opt/iviss/.env` and injects into containers

**Security:**

- `.env` file has `0600` permissions (owner read/write only)
- JSON vars file deleted after deployment
- Secrets never logged or committed to git

---

## Application Deployment on EC2

### File Structure on EC2

```
/opt/iviss/
├── docker-compose.yml    (copied from repo)
└── .env                  (generated from secrets)

# No source code on EC2!
# All code is inside Docker images pulled from GHCR
```

### Docker Images Used

**Production Images:**

- `ghcr.io/skyengpro/iviss/backend:latest`
- `ghcr.io/skyengpro/iviss/frontend:latest`
- `postgres:15-alpine`

**Image Pull:**

- Ansible logs into GHCR using GitHub credentials
- Pulls latest images with `docker compose pull`
- Images contain all compiled code

### Container Startup

**Command:** `docker compose --profile prod up -d --force-recreate`

**Flags:**

- `--profile prod`: Use production containers (not dev)
- `-d`: Detached mode (background)
- `--force-recreate`: Always recreate containers (ensures updates apply)

**Services Started:**

1. PostgreSQL database
2. Backend API (production build)
3. Frontend SPA (production build with Nginx)

**Data Persistence:**

- Database data: Docker volume `postgres_data`
- Volume persists across deployments (data not lost)

---

## SSL/TLS Configuration

### Certificate Management

**Provider:** Let's Encrypt (free SSL certificates)

**Tool:** Certbot with Nginx plugin

**Process:**

1. Ansible checks if certificate exists
2. If not exists: Runs `certbot --nginx` to obtain certificate
3. If exists: Skips (certificates auto-renew)

**Certificate Location:** `/etc/letsencrypt/live/<domain>/`

**Auto-Renewal:** Certbot sets up automatic renewal (systemd timer)

### Nginx Configuration

**Template:** `infra/ansible/roles/iviss/templates/nginx.conf.j2`

**Features:**

- HTTP → HTTPS redirect
- Reverse proxy to backend (port 3000)
- Static file serving for frontend
- SSL/TLS termination
- Security headers

**Configuration Location:** `/etc/nginx/sites-available/iviss`

---

## Ansible Roles and Tasks

### Role Structure

```
infra/ansible/roles/iviss/
├── tasks/
│   ├── main.yml          (orchestrates other tasks)
│   ├── setup.yml         (system setup)
│   ├── ssl.yml           (SSL certificate)
│   └── deploy.yml        (application deployment)
└── templates/
    ├── dotenv.j2         (main .env file)
    ├── backend.env.j2    (backend-specific env)
    ├── frontend.env.j2   (frontend-specific env)
    └── nginx.conf.j2     (Nginx configuration)
```

### Task Breakdown

**setup.yml:**

- Update apt cache
- Install Docker, Nginx, UFW, Certbot, Python packages
- Add Docker repository
- Start and enable Docker service
- Configure UFW firewall (allow 22, 80, 443)
- Create `/opt/iviss` directory
- Deploy Nginx configuration

**ssl.yml:**

- Check if SSL certificate exists
- Obtain Let's Encrypt certificate (if needed)
- Configure Nginx for HTTPS

**deploy.yml:**

- Login to GitHub Container Registry
- Copy `docker-compose.yml` to `/opt/iviss/`
- Generate `.env` file from template
- Pull latest Docker images
- Start containers with `--force-recreate`

---

## Update Process

### What Happens on Each Deployment

| Component          | Action          | Notes                                     |
| ------------------ | --------------- | ----------------------------------------- |
| AWS Infrastructure | Check/Update    | Only changes if Terraform config modified |
| System Packages    | Install Missing | Idempotent (doesn't reinstall existing)   |
| SSL Certificate    | Check/Obtain    | Skips if already exists                   |
| docker-compose.yml | Overwrite       | Always updated with latest version        |
| .env File          | Regenerate      | Always updated with latest secrets        |
| Docker Images      | Pull Latest     | Downloads new images from GHCR            |
| Containers         | Recreate        | `--force-recreate` ensures fresh start    |
| Database Data      | Preserve        | Volume persists (data not lost)           |

### Zero-Downtime Deployment

**Current State:** ❌ Not implemented

**Behavior:**

- Containers stopped
- New containers started
- ~10-30 seconds downtime during container recreation

**Future Enhancement:**

- Blue-green deployment
- Rolling updates
- Health check before traffic switch

---

## Monitoring and Health Checks

### Container Health Checks

**Backend:**

```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:3000/api/v1/health || exit 1"]
  interval: 10s
  timeout: 5s
  retries: 5
  start_period: 20s
```

**Frontend:**

```yaml
healthcheck:
  test: ["CMD-SHELL", "nc -z localhost 80 || exit 1"]
  interval: 15s
  timeout: 5s
  retries: 3
```

**Database:**

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB}"]
  interval: 10s
  timeout: 5s
  retries: 5
  start_period: 30s
```

### Missing Production Monitoring

- ❌ Application Performance Monitoring (APM)
- ❌ Log aggregation (ELK, Loki)
- ❌ Metrics collection (Prometheus in production)
- ❌ Alerting (PagerDuty, Opsgenie)
- ❌ Uptime monitoring (Pingdom, UptimeRobot)

---

## Rollback Strategy

### Current Rollback Process

**Manual Rollback:**

1. Identify last working commit
2. Revert code changes in git
3. Push to main branch
4. Wait for CI/CD to rebuild and redeploy

**Duration:** ~15-20 minutes

### Image-Based Rollback (Recommended)

**Not Implemented Yet:**

- Tag images with semantic versions
- Keep last N production images
- One-click rollback to previous image tag
- Update docker-compose.yml to use specific tag

**Example:**

```bash
# Current (always uses latest)
image: ghcr.io/skyengpro/iviss/backend:latest

# Recommended (pinned version)
image: ghcr.io/skyengpro/iviss/backend:v1.2.3
```

---

## Database Management

### Migration Execution

**When:** Backend container startup

**Tool:** SQLx migrations

**Process:**

1. Backend starts
2. Checks `_sqlx_migrations` table
3. Applies pending migrations
4. Application starts

**Location:** `iviss-backend/migrations/`

### Backup Strategy

**Current State:** ❌ Not implemented

**Recommended:**

- Daily automated backups to S3
- Point-in-time recovery
- Backup retention policy (30 days)
- Backup testing (monthly restore test)

**Implementation:**

```bash
# Cron job on EC2
0 2 * * * docker exec iviss-db pg_dump -U $POSTGRES_USER $POSTGRES_DB | gzip > /backups/db-$(date +\%Y\%m\%d).sql.gz
```

---

## Cost Analysis

### Current Monthly Costs

| Service                | Cost              | Notes                  |
| ---------------------- | ----------------- | ---------------------- |
| AWS Lightsail Instance | $12-15            | small_3_0 bundle       |
| Static IP              | $0                | Included with instance |
| Data Transfer          | $0-5              | First 1TB free         |
| S3 (Terraform state)   | <$1               | Minimal storage        |
| DynamoDB (state lock)  | <$1               | On-demand pricing      |
| **Total**              | **~$15-20/month** |                        |

### GitHub Costs

| Service        | Cost | Notes                             |
| -------------- | ---- | --------------------------------- |
| GitHub Actions | $0   | Within free tier (2000 min/month) |
| GHCR Storage   | $0   | Within free tier (500MB)          |

**Total Infrastructure Cost:** ~$15-20/month

---

## Security Posture

### Implemented Security Measures

✅ **Infrastructure:**

- Firewall (UFW) with minimal open ports
- SSH key-based authentication (no passwords)
- Static IP with firewall rules
- SSL/TLS encryption (Let's Encrypt)

✅ **Application:**

- Secrets in environment variables (not in code)
- JWT RS256 authentication
- Argon2 password hashing
- CORS configuration
- Rate limiting

✅ **CI/CD:**

- Gitleaks secret scanning
- Cargo audit (dependency vulnerabilities)
- SonarQube code analysis

### Security Gaps

❌ **Missing:**

- Container image scanning (Trivy, Snyk)
- Runtime security monitoring
- Intrusion detection system (IDS)
- Web Application Firewall (WAF)
- DDoS protection
- Security audit logs
- Penetration testing
- Compliance certifications

---

## Disaster Recovery

### Current State

**RTO (Recovery Time Objective):** ~30-60 minutes

- Requires manual intervention
- Rebuild from Terraform + Ansible
- Restore database from backup (if exists)

**RPO (Recovery Point Objective):** Undefined

- No automated backups
- Potential data loss since last manual backup

### Recommended DR Plan

1. **Automated Backups:**
   - Daily database backups to S3
   - Backup retention: 30 days
   - Cross-region replication

2. **Infrastructure as Code:**
   - ✅ Already implemented (Terraform)
   - Can rebuild infrastructure in minutes

3. **Disaster Recovery Runbook:**
   - Document recovery procedures
   - Test recovery quarterly
   - Define RTO/RPO targets

4. **Multi-Region Deployment:**
   - Active-passive setup
   - Automatic failover
   - Database replication

---

## Operational Procedures

### Deployment Checklist

**Before Deployment:**

- [ ] Review code changes
- [ ] Check CI/CD pipeline status
- [ ] Verify all tests pass
- [ ] Review database migrations
- [ ] Notify team of deployment

**During Deployment:**

- [ ] Monitor GitHub Actions workflow
- [ ] Watch for Terraform errors
- [ ] Check Ansible playbook output
- [ ] Verify containers start successfully

**After Deployment:**

- [ ] Check application health endpoints
- [ ] Verify frontend loads
- [ ] Test critical user flows
- [ ] Monitor error logs
- [ ] Confirm database migrations applied

### Troubleshooting Guide

**Issue:** Deployment fails at Terraform stage

**Possible Causes:**

- AWS credentials expired
- Terraform state locked
- Resource quota exceeded

**Resolution:**

- Check AWS credentials in GitHub Secrets
- Unlock Terraform state: `terraform force-unlock <lock-id>`
- Review AWS Lightsail quotas

---

**Issue:** Deployment fails at Ansible stage

**Possible Causes:**

- SSH connection timeout
- Docker installation failed
- Secrets not passed correctly

**Resolution:**

- Check instance is running and accessible
- SSH manually to debug: `ssh -i iviss-key.pem ubuntu@<ip>`
- Review Ansible logs in GitHub Actions

---

**Issue:** Containers fail to start

**Possible Causes:**

- Missing environment variables
- Database migration failed
- Port conflicts

**Resolution:**

- Check `.env` file on EC2: `cat /opt/iviss/.env`
- View container logs: `docker compose logs -f`
- Check port availability: `netstat -tulpn`

---

**Issue:** SSL certificate not obtained

**Possible Causes:**

- Domain not pointing to instance IP
- Port 80/443 blocked
- Rate limit exceeded (Let's Encrypt)

**Resolution:**

- Verify DNS: `dig <domain>`
- Check firewall: `sudo ufw status`
- Wait 1 hour and retry (rate limit)

---

## Future Enhancements

### Short Term (1-3 months)

1. ✅ ~~Implement production infrastructure~~ (DONE)
2. ✅ ~~Automate deployment pipeline~~ (DONE)
3. ✅ ~~Set up SSL/TLS~~ (DONE)
4. [ ] Implement database backups
5. [ ] Add production monitoring (Prometheus + Grafana)
6. [ ] Set up log aggregation
7. [ ] Implement alerting

### Medium Term (3-6 months)

1. [ ] Zero-downtime deployments
2. [ ] Container image scanning
3. [ ] Automated rollback mechanism
4. [ ] Staging environment
5. [ ] Load testing and performance tuning
6. [ ] Security audit and penetration testing

### Long Term (6-12 months)

1. [ ] Multi-region deployment
2. [ ] High availability setup
3. [ ] Auto-scaling
4. [ ] CDN integration
5. [ ] Advanced monitoring and observability
6. [ ] Chaos engineering

---

## Key Differences from Previous Documentation

### What Was Documented (April 2026 - Initial)

- ❌ No production infrastructure
- ❌ No deployment automation
- ❌ No Infrastructure as Code
- ❌ No secrets management
- ❌ Manual deployment only

### What Exists Now (April 17, 2026 - Current)

- ✅ Production infrastructure (AWS Lightsail)
- ✅ Automated deployment (GitHub Actions)
- ✅ Infrastructure as Code (Terraform)
- ✅ Configuration Management (Ansible)
- ✅ Secrets management (GitHub Secrets → Ansible)
- ✅ SSL/TLS (Let's Encrypt)
- ✅ Reverse proxy (Nginx)
- ✅ Firewall (UFW)

**Deployment Maturity:** Level 1 → Level 3 (Major improvement!)

---

## Quick Reference

### Production URLs

**Application:**
- Frontend: [https://iviss-prod.vpn.kivoyo.com/](https://iviss-prod.vpn.kivoyo.com/)
- Backend API: [https://iviss-prod.vpn.kivoyo.com/api](https://iviss-prod.vpn.kivoyo.com/api)
- Health Check: [https://iviss-prod.vpn.kivoyo.com/api/v1/health](https://iviss-prod.vpn.kivoyo.com/api/v1/health)

**Infrastructure:**
- AWS Region: eu-west-1 (Ireland)
- Domain: iviss-prod.vpn.kivoyo.com
- SSL/TLS: Let's Encrypt (automatic renewal)

### Deployment Commands

**Deploy to Production:**

```bash
# Automatic (push to main)
git push origin main

# Manual (using deploy script)
cd infra
./scripts/deploy.sh "your-domain.com" "admin@example.com"
```

**Check Deployment Status:**

```bash
# SSH into instance
ssh -i infra/ansible/iviss-key.pem ubuntu@<instance-ip>

# Check containers
docker ps

# View logs
docker compose logs -f

# Check .env file
cat /opt/iviss/.env
```

**Rollback:**

```bash
# Revert code
git revert <commit-hash>
git push origin main

# Or deploy specific version
# (requires manual docker-compose.yml edit on EC2)
```

### Important URLs

**Production:**

- Frontend: `https://<domain>` or `http://<instance-ip>:8080`
- Backend API: `https://<domain>/api` or `http://<instance-ip>:3000`
- Health Check: `http://<instance-ip>:3000/api/v1/health`

**AWS Console:**

- Lightsail: https://lightsail.aws.amazon.com/
- S3 (Terraform state): https://s3.console.aws.amazon.com/

**GitHub:**

- Actions: https://github.com/<owner>/iviss/actions
- Packages: https://github.com/<owner>/iviss/packages
- Secrets: https://github.com/<owner>/iviss/settings/secrets/actions

---

## Contact and Support

**For Deployment Issues:**

- Check GitHub Actions logs
- Review Ansible output
- SSH into instance for debugging

**For Infrastructure Changes:**

- Modify Terraform files in `infra/terraform/`
- Test locally with `terraform plan`
- Apply with `terraform apply`

**For Application Updates:**

- Push code to `main` branch
- CI/CD handles the rest automatically

---

## Document Revision History

| Version | Date           | Author          | Changes                                |
| ------- | -------------- | --------------- | -------------------------------------- |
| 1.0     | April 2026     | DevOps Analysis | Initial assessment (no infrastructure) |
| 2.0     | April 17, 2026 | DevOps Update   | Complete infrastructure implemented    |

---

**Status:** ✅ Production deployment infrastructure is fully operational and automated.
