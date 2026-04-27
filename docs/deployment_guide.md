# IVISS Master Deployment & Infrastructure Guide (v2.1)

This document provides a comprehensive, deep-dive guide for deploying and managing the IVISS platform. It covers **Infrastructure-as-Code (Terraform)**, **Configuration Management (Ansible)**, **Automated CI/CD (GitHub Actions)**, **AWS Secrets Manager**, and **OIDC Authentication**.

---

## 1. System Architecture Overview

IVISS follows a "Lean Hybrid" architecture on AWS Lightsail:
- **Compute**: Single Ubuntu 22.04 LTS Instance.
- **Hardware Profile**: **4 GB RAM, 2 vCPUs, 80 GB SSD** (Bundle: `medium_3_0`).
- **Application**: Containerized stack (Backend + Frontend + Postgres).
- **Storage/Cache**: 
  - **Postgres**: Runs in a persistent Docker container with a 10GB volume.
  - **Cache**: Handled internally by the Rust backend using **Moka** (In-memory).
- **Networking**: Lightsail Static IP with a secured Firewall (22, 80, 443).
- **API Connectivity**: Frontend uses relative paths (`/api`) proxied by Nginx.
- **Secrets**: All sensitive data stored in **AWS Secrets Manager** (not environment variables).
- **Authentication**: CI/CD uses **OIDC Assume Role** (no static AWS keys).

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
- **Lightsail**: Full access (instance management)
- **S3**: Read/write to the Terraform state bucket
- **DynamoDB**: State locking operations
- **Secrets Manager**: Read secrets for deployment

### Branch Restrictions
Only `main`, `dev`, `aws-dev-sync`, and `aws-dev-test` branches are authorized to assume the deployment role.

---

## 6. SSH & Key Management

Ansible requires an SSH key to configure the instance.
1.  **Key File**: The deployment script automatically handles a key named `iviss-key.pem`.
2.  **Auto-Cleanup**: The deploy script uses a trap handler to delete `iviss-key.pem` and `.deploy-vars.json` on exit (even on failure).
3.  **SSH Hardening**: The Ansible playbook automatically:
    - Disables password authentication
    - Prevents root login
    - Limits auth attempts to 3
    - Sets idle timeout to 5 minutes

---

## 7. Automated Deployment (GitHub Actions CI/CD)

The automated pipeline triggers on every push to `dev` or `main`. It builds the images, pushes them to GitHub Container Registry (GHCR), and triggers the Ansible deployment on the server.

### GitHub Variables (Non-Secret Configuration)

Set these in **GitHub Settings → Variables → Repository Variables**:

| Variable Name | Example Value | Description |
| :--- | :--- | :--- |
| `DOMAIN_NAME` | `iviss.example.com` | Production domain |
| `CERTBOT_EMAIL` | `admin@example.com` | SSL certificate email |
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

1.  **Get Static IP**: Find the IP in the Lightsail console or from the Terraform output.
2.  **Update Records**: Add an `A Record` in your DNS provider pointing to that IP.
3.  **SSL Generation**: The first deployment will automatically request a certificate from Let's Encrypt using the `CERTBOT_EMAIL`.
4.  **Auto-Healing**: The infrastructure includes an "Auto-Healing" task that will automatically restore your SSL configuration if Nginx is ever reinstalled or overwritten.
5.  **Security Headers**: Nginx is configured with security headers (X-Frame-Options, CSP, HSTS, etc.).

---

## 10. Operational Manual

### Logs & Monitoring
To see live application logs on the server:
```bash
cd /opt/iviss
docker compose logs -f
```

### Restarting the Stack
If you need to force a restart without a full CI/CD run:
```bash
cd /opt/iviss
docker compose down
docker compose up -d
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
- **Cause**: Nginx hasn't been configured for SSL yet or Vite API URL is wrong.
- **Fix**: Ensure `DOMAIN_NAME` is set in GitHub and `deploy.sh` runs successfully.

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
**Version 2.2 | Last Updated: 2026-04-27**
