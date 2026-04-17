# IVISS Master Deployment & Infrastructure Guide (v1.5)

This document provides a comprehensive, deep-dive guide for deploying and managing the IVISS platform. It covers **Infrastructure-as-Code (Terraform)**, **Configuration Management (Ansible)**, **Automated CI/CD (GitHub Actions)**, and **Manual Operations**.

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

---

## 2. Prerequisites

Before you begin, ensure you have:
1.  **AWS CLI** configured with an IAM user having:
    - `AmazonLightsailFullAccess`
    - `AmazonS3FullAccess` (for Terraform state)
    - `AmazonDynamoDBFullAccess` (for state locking)
2.  **Terraform** installed (v1.5+).
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
./infra/scripts/deploy.sh
```

---

## 4. SSH & Key Management

Ansible requires an SSH key to configure the instance.
1.  **Key File**: The deployment script automatically handles a key named `iviss-key.pem`.
2.  **GitHub Secrets**: For CI/CD, you don't need the PEM file; the `deploy-aws.yml` uses the AWS API to interact with the instance.

---

## 5. Automated Deployment (GitHub Actions CI/CD)

The automated pipeline triggers on every push to `dev` or `main`. It builds the images, pushes them to GitHub Container Registry (GHCR), and triggers the Ansible deployment on the server.

### Mandatory Secrets Checklist (21 Total):

| Secret Name | Category | Description |
| :--- | :--- | :--- |
| **AWS_ACCESS_KEY_ID** | AWS | Your IAM Access Key |
| **AWS_SECRET_ACCESS_KEY** | AWS | Your IAM Secret Key |
| **DOMAIN_NAME** | Connectivity | e.g. `yourdomain.com` |
| **CERTBOT_EMAIL** | Connectivity | Real email address for SSL alerts |
| **JWT_SECRET** | Auth | Secure random string |
| **JWT_PRIVATE_KEY_PEM** | Auth | RS256 Private Key (`cat jwt-private.pem`) |
| **JWT_PUBLIC_KEY_PEM** | Auth | RS256 Public Key (`cat jwt-public.pem`) |
| **ACTIVATION_CODE_PEPPER**| Auth | Random 64-char pepper |
| **SHIFT_START_HOUR** | Policy | e.g. `6` (Start of operations) |
| **SHIFT_END_HOUR** | Policy | e.g. `18` (End of operations) |
| **RUST_LOG** | Logging | e.g. `info` or `debug` |
| **REGISTRY_USERNAME** | GHCR | Your GitHub Username |
| **REGISTRY_TOKEN** | GHCR | Your PAT (with `read:packages`) |
| **ADMIN_BOOTSTRAP_EMAIL** | Admin | Initial admin email |
| **ADMIN_BOOTSTRAP_PASSWORD**| Admin | Initial admin password |
| **ADMIN_BOOTSTRAP_PHONE** | Admin | Initial admin phone |
| **ADMIN_BOOTSTRAP_USERNAME**| Admin | Initial admin username |
| **POSTGRES_USER** | Database | e.g. `iviss_user` |
| **POSTGRES_PASSWORD** | Database | A secure DB password |
| **POSTGRES_DB** | Database | e.g. `iviss_prod` |
| **SMS_PROVIDER** | Provider | `mock`, `twilio`, or `vonage` |
| **VONAGE_API_KEY** | SMS | Required if provider is `vonage` |
| **VONAGE_API_SECRET** | SMS | Required if provider is `vonage` |
| **TWILIO_ACCOUNT_SID** | SMS | Required if provider is `twilio` |
| **TWILIO_AUTH_TOKEN** | SMS | Required if provider is `twilio` |
| **TWILIO_FROM_NUMBER** | SMS | Required if provider is `twilio` |
| **EMAIL_PROVIDER** | Provider | `mock`, `resend`, or `smtp` |
| **RESEND_API_KEY** | Email | Required if provider is `resend` |
| **RESEND_FROM_EMAIL** | Email | e.g. `iviss@iviss.com` |
| **SMTP_HOST** | Email | Required if provider is `smtp` |
| **SMTP_PORT** | Email | e.g. `587` |
| **SMTP_USERNAME** | Email | SMTP Login |
| **SMTP_PASSWORD** | Email | SMTP Password |
| **SMTP_FROM_EMAIL** | Email | Valid sender address |
| **VITE_API_URL** | Web | `https://yourdomain.com` |

---

## 6. Provider Configuration

IVISS supports multiple providers for SMS and Email. These are toggled via the `*_PROVIDER` environment variables.

### A. SMS Providers
- **`mock`** (Default): Logs OTP codes directly to the backend console (no carrier fees).
- **`twilio`**: Uses standard Twilio REST API. Requires `TWILIO_*` variables.
- **`vonage`**: Uses Vonage/Nexmo API. Requires `VONAGE_*` variables.

### B. Email Providers
- **`mock`**: Logs email content to the backend console.
- **`resend`**: Uses Resend.com API (High delivery speed). Requires `RESEND_*` variables.
- **`smtp`**: Uses standard SMTP protocol (for Outlook, Gmail, or custom relays). Requires `SMTP_*` variables.

---

## 7. DNS & SSL Setup

1.  **Get Static IP**: Find the IP in the Lightsail console or from the Terraform output.
2.  **Update Records**: Add an `A Record` in your DNS provider pointing to that IP.
3.  **SSL Generation**: The first deployment will automatically request a certificate from Let's Encrypt using the `CERTBOT_EMAIL`.
4.  **Auto-Healing**: The infrastructure includes an "Auto-Healing" task that will automatically restore your SSL configuration if Nginx is ever reinstalled or overwritten.

---

## 8. Operational Manual

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

## 9. Security: Manual Key Generation

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

### C. Formatting for .env
To get the single-line string with `\n` needed for the `.env` file:
```bash
awk '{printf "%s\\n", $0}' jwt-private.pem
```

---

## 10. Troubleshooting


### 1. "Unauthorized" or "401" on Frontend
- **Cause**: JWT Key mismatch or expired session.
- **Fix**: Ensure `JWT_PRIVATE_KEY_PEM` matches the version used locally.

### 2. "Conflict: Target already exists" (Terraform)
- **Cause**: Trying to create an instance that already exists.
- **Fix**: Use `terraform import` or ensure you are using the correct workspace/remote state.

### 3. File Size Limit (Push Failed)
- **Cause**: Accidentally committed `.terraform/` binary files.
- **Fix**: Run `git reset --soft HEAD~1`, then `git rm -r --cached infra/terraform/.terraform/`, update `.gitignore`, and re-commit.

---

