# IVISS Master Deployment & Infrastructure Guide (v1.1)

This document provides a comprehensive, deep-dive guide for deploying and managing the IVISS platform. It covers **Infrastructure-as-Code (Terraform)**, **Configuration Management (Ansible)**, **Automated CI/CD (GitHub Actions)**, and **Manual Operations**.

---

## 1. System Architecture Overview

IVISS follows a "Lean Hybrid" architecture on AWS Lightsail:
- **Compute**: Single Ubuntu 22.04 LTS Instance.
- **Hardware Profile**: **2 GB RAM, 2 vCPUs, 60 GB SSD** (Bundle: `small_3_0`).
- **Application**: Containerized stack (Backend + Frontend + Postgres + Metrics).
- **Storage/Cache**: Postgres runs in a persistent Docker container. High-speed caching is handled internally by the Rust backend using **Moka**, utilizing the server's 2GB RAM directly without needing a separate Redis instance.
- **Networking**: Lightsail Static IP with a secured Firewall (22, 80, 443).
- **API Connectivity**: Frontend uses relative paths (`/api`) proxied by Nginx.

---

## 2. Infrastructure Layer (Terraform & Remote State)

### State Management (Remote Backend):
We store our Terraform state in an **S3 Bucket** with **DynamoDB Locking**.
- **Storage**: S3 Bucket (Versioned & Encrypted).
- **Locking**: DynamoDB Table (`iviss-terraform-lock`).
- **Initialization**: Run `./infra/scripts/setup-remote-state.sh` **once** to provision this storage.

---

## 5. Automated Deployment (GitHub Actions CI/CD)

### Mandatory Secrets Checklist (13 Total):
| Secret Name | Category | Description |
| :--- | :--- | :--- |
| **AWS_ACCESS_KEY_ID** | AWS | Your IAM Access Key |
| **AWS_SECRET_ACCESS_KEY** | AWS | Your IAM Secret Key |
| **DOMAIN_NAME** | Connectivity | e.g. `iviss.vpn.kivoyo.com` |
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
| **TWILIO_ACCOUNT_SID** | SMS | Twilio Account SID (or `mock`) |
| **TWILIO_AUTH_TOKEN** | SMS | Twilio Auth Token (or `mock`) |
| **TWILIO_FROM_NUMBER** | SMS | Twilio Phone Number (or `mock`) |


---

## 6. Environment Variable Management (.env)

#### Local Development Setup
1. **Copy Example**: `cp .env.example .env`
2. **Generate Tokens**: 
   - **JWT Secrets**: `openssl rand -base64 32`
   - **Peppers**: `openssl rand -base64 48 | cut -c1-64`
   - **RS256 Keys**:
     ```bash
     openssl genrsa -out jwt-private.pem 2048
     openssl rsa -in jwt-private.pem -outform PEM -pubout -out jwt-public.pem
     ```
3. **Internal DB**: You can leave the local Postgres and Redis settings as defaults for your dev machine.

---

## 7. Operational Manual

### Logs & Monitoring
```bash
cd /opt/iviss
docker compose logs -f
```

### Portability Reminder
The frontend is domain-agnostic. It calls `/api`, which Nginx (on the server host) redirects to the backend container. **Never hardcode "localhost:3000" or IPs in the frontend code.**

---

*Last Updated: April 2026*
