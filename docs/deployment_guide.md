# IVISS Master Deployment & Infrastructure Guide (v3.3)

This document provides a comprehensive, deep-dive guide for deploying and managing the IVISS platform. It covers **Infrastructure-as-Code (Terraform)**, **Configuration Management (Ansible)**, **Automated CI/CD (GitHub Actions)**, **AWS Secrets Manager**, and **OIDC Authentication**.

**Latest Changes (v3.3 - 2026-04-30):**
- Edge lockdown enabled by default for security-first deployments
- CloudFront cache policy modernization (AWS-managed policies)
- Conditional Certbot installation (only when CloudFront disabled)
- Improved Nginx origin header validation
- Relay/EICE access path removed
- CloudFront origin lockdown now uses AWS-published `CLOUDFRONT_ORIGIN_FACING` ranges
- Temporary debug mode opens SSH for validation/troubleshooting

---

## 1. System Architecture Overview

IVISS follows a cost-aware edge-and-origin architecture on AWS:

### Traffic Flow
```
Internet → CloudFront (HTTPS + WAF) → Lightsail Origin (HTTP + Nginx) → App Containers
                                                              ↓
                                           Debug SSH (public) → Lightsail Public IP
```

### Components
- **Edge Layer**: Amazon CloudFront + AWS WAF as the public HTTPS entrypoint
  - TLS termination at CloudFront using ACM certificate (us-east-1)
  - AWS WAF with managed rule groups (Common + KnownBadInputs)
  - Custom origin verification header (`X-Origin-Verify`)
  - Modern cache policies (CachingDisabled for dynamic content)

- **Origin Layer**: Single Ubuntu 22.04 LTS Amazon Lightsail instance
  - Nginx reverse proxy with origin header validation
  - Docker Compose stack (Backend + Frontend + PostgreSQL)
  - UFW host firewall (SSH open for debugging/testing mode)
  - HTTP-only from CloudFront (Phase 1)

- **Access Layer**: Temporary debug administrative SSH path
  - Deploy script opens Lightsail SSH (`22/tcp`) to `0.0.0.0/0` for setup/testing
  - UFW currently allows SSH ingress for debugging/testing
  - Revert to CIDR-restricted SSH after validation is complete

- **Application Stack**:
  - **Backend**: Rust + Axum (port 3000)
  - **Frontend**: React + Vite (port 8080)
  - **Database**: PostgreSQL 15 (persistent Docker volume)
  - **Cache**: Moka in-memory cache (Rust backend)

- **Security**:
  - CloudFront-origin CIDR restriction on Lightsail firewall
  - Origin secret stored in AWS Secrets Manager
  - OIDC-based GitHub Actions authentication (no static keys)
  - Security-first defaults (edge lockdown enabled)

- **API Connectivity**: Frontend uses relative paths (`/api`) proxied by Nginx
- **Secrets**: All sensitive data stored in **AWS Secrets Manager**

---

## 2. Prerequisites

Before you begin, ensure you have:
1.  **AWS CLI** configured with credentials that have:
    - `AmazonLightsailFullAccess`
    - `AmazonS3FullAccess` (for Terraform state)
    - `AmazonDynamoDBFullAccess` (for state locking)
    - `SecretsManagerReadWrite` (for secret management)
2.  **Terraform** installed (v1.7+).
3.  **Ansible** installed (for manual deployment fallback).
4.  **GitHub PAT** with `write:packages` and `read:packages` permissions.

---

## 3. Infrastructure Layer (Terraform & Remote State)

### State Management (Remote Backend)
We store our Terraform state in an **S3 Bucket** with **DynamoDB Locking** to prevent corruption during team deployments.

#### A. Initial Setup (One-time only)
Run this script to provision the S3 and DynamoDB storage:
```bash
chmod +x ./infra/scripts/setup-remote-state.sh
./infra/scripts/setup-remote-state.sh
```

#### B. Provisioning the Instance
To create the Lightsail instance and static IP:
```bash
cd infra/terraform
terraform init
terraform apply -var="domain_name=yourdomain.com"
```
Or use the wrapper script:
```bash
cd infra/
export ROUTE53_ZONE_ID=ZXXXXXXXXXXXXX
export EDGE_LOCKDOWN_ENABLED=true
export IMAGE_TAG=latest
export USE_SECRETS_MANAGER=true
./scripts/deploy.sh
```

### C. Infrastructure Teardown
To completely remove the provisioned infrastructure (Instance, Ports, Static IP attachment), use the destroy script:

```bash
cd infra/
./scripts/destroy.sh
```

> [!CAUTION]
> This will permanently delete the Lightsail instance and its data. It will **not** delete the Static IP or the Terraform State bucket, allowing you to re-deploy to the same entry point later.

---

## 4. Secrets Management (AWS Secrets Manager)

All sensitive data is stored in AWS Secrets Manager, organized into two groups:

### Secret Groups

| Secret Name | Contents | When to Rotate |
| :--- | :--- | :--- |
| `iviss/<env>/app-secrets` | JWT keys, DB password, admin password, activation pepper, GHCR token | On suspected compromise or every 90 days |
| `iviss/<env>/provider-keys` | Twilio, Vonage, Resend, SMTP credentials | When rotating provider keys |
| `iviss/<env>/cloudfront-origin-secret` | Shared `X-Origin-Verify` header value between CloudFront and Nginx | On suspected origin leak or every 90 days |

### A. Initial Secret Seeding (One-time)

After running `terraform apply` for the first time (which creates empty secrets), populate them:

```bash
# Group 1: App Secrets
aws secretsmanager put-secret-value \
  --region <AWS_REGION> \
  --secret-id "iviss/production/app-secrets" \
  --secret-string '{
    "jwt_private_key_pem": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----",
    "jwt_public_key_pem": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
    "activation_code_pepper": "your_64_char_pepper",
    "db_password": "secure_postgres_pass",
    "admin_bootstrap_password": "admin_portal_pass",
    "docker_password": "your_github_pat"
  }'

# Group 2: Provider Keys (SMS & Email)
aws secretsmanager put-secret-value \
  --region <AWS_REGION> \
  --secret-id "iviss/production/provider-keys" \
  --secret-string '{
    "twilio_account_sid": "AC...",
    "twilio_auth_token": "...",
    "twilio_from_number": "+1...",
    "vonage_api_key": "...",
    "vonage_api_secret": "...",
    "orange_client_id": "...",
    "orange_client_secret": "...",
    "orange_sender_number": "...",
    "resend_api_key": "re_...",
    "smtp_password": "..."
  }'
```

### B. Rotating a Secret

```bash
# Example: rotate the DB password
aws secretsmanager get-secret-value \
  --secret-id "iviss/production/app-secrets" \
  --query SecretString --output text | \
  python3 -c "
import json, sys
s = json.load(sys.stdin)
s['db_password'] = 'new_secure_password_here'
print(json.dumps(s))
" | \
  aws secretsmanager put-secret-value \
    --secret-id "iviss/production/app-secrets" \
    --secret-string file:///dev/stdin
```

### C. How Secrets Flow

```
AWS Secrets Manager
       │
       ▼ (fetched at deploy time by deploy.sh)
  .deploy-vars.json  ← Temporary file, auto-deleted after deploy
       │
       ▼ (passed to Ansible via --extra-vars)
  Ansible Playbook
       │
       ▼ (rendered into templates)
  /opt/iviss/.env  ← On production server, mode 0600
```

---

## 5. AWS Authentication (OIDC Assume Role)

The CI/CD pipeline uses **GitHub Actions OIDC** to authenticate with AWS — no static access keys required.

### How It Works
1. GitHub Actions requests a short-lived OIDC token from GitHub's identity provider
2. AWS STS validates the token against the configured OIDC provider
3. AWS issues temporary credentials (1-hour TTL) scoped to the deploy IAM role
4. The deploy script runs with these temporary credentials

### IAM Role ARN
```
arn:aws:iam::<YOUR_ACCOUNT_ID>:role/iviss-github-actions-deploy
```

### Permissions Granted
- **Lightsail**: Full access (instance management and firewall updates)
- **CloudFront**: Distribution CRUD, cache invalidation
- **WAFv2**: Web ACL management (CloudFront scope)
- **ACM**: Certificate management (us-east-1)
- **Route53**: DNS record management for CloudFront alias
- **S3**: Read/write to the Terraform state bucket
- **DynamoDB**: State locking operations
- **Secrets Manager**: Read/write for app secrets and origin verification key

### Branch Restrictions
Only `main`, `dev`, `aws-dev-sync`, and `aws-dev-test` branches are authorized to assume the deployment role.

---

## 6. SSH & Key Management

Administrative access now uses a temporary debug direct path:
1.  **Admin Path**: `operator/GitHub Actions -> Lightsail public IP`.
2.  **Key Files**: The deployment script materializes `iviss-key.pem` only, then removes it on exit.
3.  **Public SSH**: Lightsail port `22` is opened to `0.0.0.0/0` during debugging/testing.
4.  **Host Firewall**: UFW allows SSH ingress for debugging/testing.

---

## 7. Automated Deployment (GitHub Actions CI/CD)

The automated pipeline builds the images, pushes them to GitHub Container Registry (GHCR), and triggers deployment using the same script flow.

### GitHub Variables (Non-Secret Configuration)

Set these in **GitHub Settings → Variables → Repository Variables**:

| Variable Name | Example Value | Description |
| :--- | :--- | :--- |
| `DOMAIN_NAME` | `iviss.example.com` | Production domain |
| `ROUTE53_ZONE_ID` | `Z0123456789ABCDEF` | Hosted zone used for ACM validation and CloudFront alias records |
| `DOCKER_USERNAME` | `yourusername` | GHCR username (Required for image pulls) |
| `AWS_ROLE_ARN` | `arn:aws:iam::<ID>:role/...` | OIDC Role ARN for AWS Auth |
| `ENVIRONMENT` | `production` | Deployment environment |
| `SMS_PROVIDER` | `orange` | `mock`, `twilio`, `vonage`, or `orange` |
| `EMAIL_PROVIDER` | `lettre` | `mock`, `resend`, or `lettre` |
| `SHIFT_START_HOUR` | `6` | Start of operations (UTC+1) |
| `SHIFT_END_HOUR` | `18` | End of operations (UTC+1) |
| `POSTGRES_USER` | `iviss_user` | Database username |
| `POSTGRES_DB` | `iviss_dev` | Database name |
| `ADMIN_BOOTSTRAP_EMAIL` | `admin@iviss.com` | Initial admin email |
| `ADMIN_BOOTSTRAP_USERNAME` | `admin` | Initial admin username |
| `SMTP_HOST` | `smtp.gmail.com` | SMTP server |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_FROM_EMAIL` | `noreply@iviss.cloud` | SMTP sender address |
| `SMTP_USERNAME` | `user@gmail.com` | SMTP username |
| `RESEND_FROM_EMAIL`| `onboarding@resend.dev`| Resend sender address |
| `AWS_REGION` | `eu-west-1` | AWS deployment region |

> **Note:** All actual secrets (passwords, API keys, tokens) are stored in AWS Secrets Manager, not GitHub Secrets.
>
> **Required for full custom-domain automation in CI/CD:** `DOMAIN_NAME`, `ROUTE53_ZONE_ID`, and `DOCKER_USERNAME`.

### GitHub Environments

| Environment | Branch | Protection |
| :--- | :--- | :--- |
| `production` | `main` | Required reviewers recommended |
| `staging` | `dev` | No protection |

---

## 8. Provider Configuration

IVISS supports multiple providers for SMS and Email. These are toggled via the `*_PROVIDER` GitHub Variables.

### A. SMS Providers
- **`mock`** (Default): Logs OTP codes directly to the backend console (no carrier fees).
- **`twilio`**: Uses standard Twilio REST API. Requires Twilio secrets in Secrets Manager.
- **`vonage`**: Uses Vonage/Nexmo API. Requires Vonage secrets in Secrets Manager.
- **`orange`**: Uses Orange SMS API. Requires Orange secrets in Secrets Manager.

### B. Email Providers
- **`mock`**: Logs email content to the backend console.
- **`resend`**: Uses Resend.com API (High delivery speed). Requires `resend_api_key` in Secrets Manager.
- **`lettre`**: Uses standard SMTP protocol (for Outlook, Gmail, or custom relays). Requires `smtp_password` in Secrets Manager.

---

## 9. DNS & SSL Setup

### TLS Architecture
1.  **Viewer TLS terminates at CloudFront** using an ACM certificate in `us-east-1`
2.  **DNS points to CloudFront**, not to the Lightsail static IP
3.  **Route53 automation** is enabled when `ROUTE53_ZONE_ID` is provided; otherwise, use the CloudFront distribution domain until DNS is wired manually

### Origin Protection (Two Layers)
4.  **Network Layer**: Lightsail public firewall only permits CloudFront origin-facing IPv4 ranges on port `80`
    - CloudFront CIDRs are synced from AWS-published IP ranges
    - Updated automatically during Terraform apply
    
5.  **Application Layer**: Nginx validates `X-Origin-Verify` header
    - Header value stored in AWS Secrets Manager
    - Requests without valid header receive `403 Forbidden`
    - Health check endpoint (`/__origin_check__`) exempt from validation

### Origin Protocol
6.  **HTTP-only from CloudFront to Lightsail (current mode)**
    - Traffic encrypted from viewer to CloudFront
    - CloudFront-to-origin stays HTTP in this setup
    - `X-Origin-Verify` header validation remains enabled as an additional origin-auth layer

### Edge Lockdown
7.  **Security-first default**: `EDGE_LOCKDOWN_ENABLED=true` by default
    - HTTP origin access remains restricted to CloudFront CIDRs
    - Only CloudFront CIDRs allowed on port 80
    - Current debug profile keeps SSH open during validation
    - Warning displayed if explicitly disabled in production

---

## 10. Operational Manual

### Deployment Commands

#### Full Production Deployment
```bash
cd infra/
# deploy.sh reads DOMAIN_NAME and other defaults from repository .env
export USE_SECRETS_MANAGER=true
export EDGE_LOCKDOWN_ENABLED=true  # Default, but explicit is better
# Required when USE_SECRETS_MANAGER=true (password comes from AWS secret app-secrets.docker_password)
# export DOCKER_USERNAME=your-ghcr-username
# Optional overrides (only if you do NOT want .env values)
# export DOMAIN_NAME=yourdomain.com
# export ROUTE53_ZONE_ID=Z0123456789ABCDEF
./scripts/deploy.sh
```

#### Pre-flight Checklist (`USE_SECRETS_MANAGER=true`)
- AWS credentials are available (`aws sts get-caller-identity` works)
- `DOCKER_USERNAME` is set (env var or `.env`)
- Secret `iviss/<env>/app-secrets` contains `docker_password`, JWT keys, `db_password`, `admin_bootstrap_password`
- Secret `iviss/<env>/provider-keys` contains provider secrets used by selected `SMS_PROVIDER` / `EMAIL_PROVIDER`
- Secret `iviss/<env>/cloudfront-origin-secret` exists

### Deployment Modes

#### Mode A — Local deploy with `.env` only
- `USE_SECRETS_MANAGER=false` (default)
- Non-sensitive + sensitive values are read from local `.env` (and local key files for JWT fallback)
- Best for local/dev or bootstrap testing

#### Mode B — Local deploy with AWS Secrets Manager + `.env`
- `USE_SECRETS_MANAGER=true`
- Sensitive values come from AWS Secrets Manager:
  - `iviss/<env>/app-secrets`
  - `iviss/<env>/provider-keys`
  - `iviss/<env>/cloudfront-origin-secret`
- Non-sensitive values still come from `.env` / exported env vars (`DOMAIN_NAME`, `DOCKER_USERNAME`, `POSTGRES_USER`, `SMS_PROVIDER`, etc.)

#### Mode C — Automated CI/CD deploy (OIDC + Secrets Manager)
- GitHub Actions assumes AWS role using OIDC (`aws-actions/configure-aws-credentials`)
- Runs the same `infra/scripts/deploy.sh`
- `USE_SECRETS_MANAGER=true` in workflow env
- Non-secret knobs from GitHub Variables; secrets remain in AWS Secrets Manager

### Why `ROUTE53_ZONE_ID` matters
- It enables Terraform to automatically:
  - create ACM DNS validation records for your custom domain certificate (in `us-east-1`)
  - create Route53 alias `A/AAAA` records pointing your domain to CloudFront
- If not set:
  - CloudFront still deploys
  - custom-domain DNS automation is skipped
  - you use CloudFront default domain or configure DNS records manually

#### Development Deployment (Edge Lockdown Disabled)
```bash
export EDGE_LOCKDOWN_ENABLED=false  # WARNING: Not recommended for production
./scripts/deploy.sh
```

### SSH Access

#### During Active Deploy Window
```bash
# SSH config is generated during deployment
ssh -F infra/ansible/ssh_config lightsail-public
```

> Current debug profile keeps SSH open; close it manually after validation.

### Logs & Monitoring

#### Application Logs
```bash
ssh -F infra/ansible/ssh_config lightsail-public
cd /opt/iviss
docker compose logs -f
docker compose logs -f backend  # Backend only
docker compose logs -f frontend  # Frontend only
```

#### Nginx Logs
```bash
ssh -F infra/ansible/ssh_config lightsail-public
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log
```

#### CloudFront Logs (if enabled)
```bash
# Logs stored in S3 bucket (configure in CloudFront settings)
aws s3 ls s3://your-cloudfront-logs-bucket/
```

#### WAF Logs
```bash
# CloudWatch Logs (configure in WAF settings)
aws logs describe-log-groups --log-group-name-prefix aws-waf-logs
```

### Restarting the Stack

#### Full Restart
```bash
ssh -F infra/ansible/ssh_config lightsail-public
cd /opt/iviss
docker compose down
docker compose up -d
```

#### Service-Specific Restart
```bash
# Restart backend only
docker compose restart backend

# Restart frontend only
docker compose restart frontend

# Restart PostgreSQL
docker compose restart db
```

### Health Checks

#### CloudFront Endpoint
```bash
curl -I https://yourdomain.com
curl -I https://yourdomain.com/api/v1/health
```

#### Direct Origin (for debugging)
```bash
# From an IP in CloudFront CIDR range
ORIGIN_SECRET=$(aws secretsmanager get-secret-value \
  --secret-id iviss/production/cloudfront-origin-secret \
  --query SecretString --output text)

curl -H "X-Origin-Verify: $ORIGIN_SECRET" http://LIGHTSAIL_IP/api/v1/health
```

### Database Operations

#### Connect to PostgreSQL
```bash
ssh -F infra/ansible/ssh_config lightsail-public
docker compose exec db psql -U iviss_user -d iviss_dev
```

#### Database Backup
```bash
ssh -F infra/ansible/ssh_config lightsail-public
docker compose exec db pg_dump -U iviss_user iviss_dev > /tmp/backup_$(date +%Y%m%d).sql
```

#### Database Restore
```bash
ssh -F infra/ansible/ssh_config lightsail-public
docker compose exec -T db psql -U iviss_user iviss_dev < /tmp/backup.sql
```

---

## 11. Security: Manual Key Generation

If you need to rotate secrets or generate new keys for a fresh environment, use these commands:

### A. JWT HMAC Secret
```bash
openssl rand -base64 48
```

### B. JWT RSA Key Pair (Private & Public)
```bash
# 1. Generate Private Key
openssl genrsa -out jwt-private.pem 2048

# 2. Extract Public Key
openssl rsa -in jwt-private.pem -pubout -out jwt-public.pem
```

### C. Formatting for Secrets Manager
To get the single-line string with `\n` needed for Secrets Manager JSON:
```bash
awk '{printf "%s\\n", $0}' jwt-private.pem
```

### D. Activation Code Pepper
```bash
openssl rand -hex 32
```

---

## 12. Troubleshooting

### 1. Terraform State Checksum Mismatch
- **Cause**: S3 state and DynamoDB lock are out of sync.
- **Fix**: Clear the DynamoDB lock entry manually.
  ```bash
  aws dynamodb delete-item \
      --table-name "iviss-terraform-lock" \
      --key '{"LockID": {"S": "YOUR_BUCKET_NAME/production/terraform.tfstate-md5"}}' \
      --region "<AWS_REGION>"
  ```

### 2. "Resource already exists" (Lightsail)
- **Cause**: Existing resources found in AWS but missing from your current `.tfstate`.
- **Fix**: Try to import them. If Terraform reports "resource doesn't support import", manually delete the resource in the AWS Console and rerun the deployment.
  ```bash
  cd infra/terraform
  terraform import aws_lightsail_instance.iviss_app iviss-production-app
  # For IP/Key - if import fails: Manually delete in Console, then 'terraform apply'
  ```

### 3. Secrets Manager "ResourceNotFoundException"
- **Cause**: Secrets haven't been created yet (first deploy).
- **Fix**: Run `terraform apply` first to create the secret resources, then seed them with the AWS CLI commands in Section 4.

### 4. Frontend showing "403" / "White Screen"
- **Cause**: CloudFront is not sending the correct origin secret header, or requests are bypassing CloudFront and hitting the origin directly
- **Fix**: 
  1. Ensure the `cloudfront-origin-secret` secret exists in AWS Secrets Manager
  2. Verify CloudFront distribution has the custom header configured:
     ```bash
     aws cloudfront get-distribution --id DISTRIBUTION_ID \
       --query 'Distribution.DistributionConfig.Origins.Items[0].CustomHeaders' 
     ```
  3. Confirm request is going through CloudFront domain, not direct IP
  4. Check Nginx error logs on Lightsail:
     ```bash
     sudo tail -50 /var/log/nginx/error.log
     ```

### 5. SSH Connection Timeout
- **Cause**: Lightsail/UFW SSH rule mismatch or host networking not ready
- **Fix**: Verify Lightsail SSH port and UFW status, then retry:
  ```bash
  aws lightsail get-instance-public-ports --region eu-west-1 --instance-name iviss-production-app-v2
  # On host:
  sudo ufw status
  ```

### 6. Terraform Plan Shows CloudFront CIDR Changes
- **Cause**: AWS updates CloudFront IP ranges periodically
- **Fix**: This is expected. Apply the changes to update Lightsail firewall:
  ```bash
  cd infra/terraform
  terraform apply -var="edge_lockdown_enabled=true"
  ```

### 7. WAF Blocking Legitimate Traffic
- **Cause**: AWSManagedRules false positive
- **Fix**: 
  1. Check WAF logs in CloudWatch
  2. Add exclusions in WAF Web ACL if needed
  3. Test with WAF in "count" mode before "block"

### 8. Certbot Installed Despite CloudFront Enabled
- **Cause**: `cloudfront_enabled` variable not passed to Ansible
- **Fix**: Ensure deploy.sh passes `cloudfront_enabled` based on `EDGE_LOCKDOWN_ENABLED`

---

## 13. Teardown & Maintenance

### A. Partial Destruction (Instance only)
To save costs while keeping your Static IP and DNS settings intact:
```bash
cd infra/terraform
terraform destroy -target=aws_lightsail_instance.iviss_app
```

### B. Full Cleanup
If you need to wipe everything including the remote state storage (S3/DynamoDB), you must manually delete the S3 bucket and DynamoDB table via the AWS CLI or Console, as these are "bootstrap" resources not managed by the main `iviss` Terraform module.

---
**Version 3.3 | Last Updated: 2026-04-30**

### Version History
- **v3.3 (2026-04-30)**: CloudFront origin-facing CIDR lockdown fix documented and CI/CD required variable notes clarified
- **v3.2 (2026-04-30)**: Temporary debug SSH profile documented
- **v3.1 (2026-04-30)**: Relay/EICE removed, temporary caller-scoped SSH deployment flow documented
- **v3.0 (2026-04-30)**: Edge lockdown by default, CloudFront cache policy modernization, conditional Certbot, improved Nginx validation
- **v2.3 (2026-04-29)**: Comprehensive architecture documentation
- **v2.1**: Initial comprehensive guide with Secrets Manager and OIDC
