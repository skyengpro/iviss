# IVISS Deployment & Infrastructure Guide

> **Who is this for?**
> This document covers everything needed to deploy, configure, and maintain the IVISS platform in a production environment. It is written for both technical administrators and informed stakeholders who need to understand how the system is hosted and updated.

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [How the Pipeline Works](#2-how-the-pipeline-works)
3. [Infrastructure — What is Provisioned](#3-infrastructure--what-is-provisioned)
4. [First-Time Setup](#4-first-time-setup)
5. [GitHub Secrets — Configuration Reference](#5-github-secrets--configuration-reference)
6. [Provider Configuration (SMS & Email)](#6-provider-configuration-sms--email)
7. [DNS & SSL (HTTPS)](#7-dns--ssl-https)
8. [Manual Operations on the Server](#8-manual-operations-on-the-server)
9. [Security — Key Generation & Rotation](#9-security--key-generation--rotation)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. System Overview

IVISS runs on **AWS Lightsail** — Amazon's simplified cloud hosting service. The entire application is packaged in **Docker containers**, which means it is self-contained, portable, and easy to update.

### Server Specifications

| Resource | Value |
|---|---|
| Cloud Provider | AWS Lightsail |
| Region | Europe (eu-west-1) |
| Operating System | Ubuntu 22.04 LTS |
| CPU | 2 vCPUs |
| RAM | 2 GB |
| Storage | 60 GB SSD |

### What runs on the server

The server runs three containers that work together:

- **Backend** — The Rust API server that handles all business logic, authentication, and data
- **Frontend** — The web application served via Nginx (the interface agents and admins use)
- **Database** — PostgreSQL, storing all vehicle records, users, and control history

---

## 2. How the Pipeline Works

Every time a developer pushes code to GitHub, an automated pipeline runs. Here is what happens step by step:

```
Developer pushes code
        │
        ▼
┌─────────────────────┐
│   CI Checks run     │  ← Tests, linting, type checks (backend & frontend)
└─────────────────────┘
        │ (if checks pass)
        ▼
┌─────────────────────┐
│  Docker images      │  ← New versions of the app are packaged
│  are built          │
└─────────────────────┘
        │ (push to main or dev)
        ▼
┌─────────────────────┐
│  Deploy to AWS      │  ← Server is updated with the new version
└─────────────────────┘
        │ (push to main only)
        ▼
┌─────────────────────┐
│  Release created    │  ← Version number assigned, changelog published
└─────────────────────┘
```

### Which branch triggers what

| Action | `dev` branch | `main` branch |
|---|---|---|
| CI checks (tests, lint) | ✅ Yes | ✅ Yes |
| Docker images built | ✅ Yes | ✅ Yes |
| Deploy to AWS | ✅ Yes | ✅ Yes |
| Release created | ❌ No | ✅ Yes (on `dev`) |

> **In plain terms:** `dev` is both the testing environment and where releases are created. `main` is production. Merging into `dev` creates an official release and triggers a deploy.

### Triggering a deployment manually

If you need to trigger a deployment without pushing new code:

1. Go to the GitHub repository
2. Click **Actions** in the top menu
3. Select **"Docker"** from the left list
4. Click **"Run workflow"** → type `yes` → click the green button

---

## 3. Infrastructure — What is Provisioned

The server infrastructure is managed using **Terraform** — a tool that creates and configures cloud resources automatically from code. This means the entire server setup is reproducible and version-controlled.

### What Terraform creates

- An AWS Lightsail instance (the virtual server)
- A static IP address (so the domain always points to the same address)
- Firewall rules (only ports 22, 80, and 443 are open)

### What Ansible configures

Once the server exists, **Ansible** (a configuration tool) connects to it and:

- Installs Docker
- Copies the application configuration
- Starts all containers
- Configures Nginx as a reverse proxy
- Sets up SSL certificates (HTTPS)

---

## 4. First-Time Setup

This section covers what needs to be done once to get the system running from scratch.

### Step 1 — Prerequisites

Make sure the following tools are installed on your local machine:

- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html) — configured with an IAM user that has Lightsail, S3, and DynamoDB access
- [Terraform](https://developer.hashicorp.com/terraform/install) (version 1.5 or higher)
- [Ansible](https://docs.ansible.com/ansible/latest/installation_guide/index.html)
- A GitHub Personal Access Token (PAT) with `read:packages` and `write:packages` permissions

### Step 2 — Set up remote state storage (one time only)

Terraform needs a place to store its state file so the team can share it safely:

```bash
chmod +x ./infra/scripts/setup-remote-state.sh
./infra/scripts/setup-remote-state.sh
```

This creates an S3 bucket and a DynamoDB table on AWS to store and lock the Terraform state.

### Step 3 — Configure GitHub Secrets

All sensitive configuration (passwords, API keys, etc.) must be added as **GitHub Secrets** in the repository settings. See [Section 5](#5-github-secrets--configuration-reference) for the full list.

### Step 4 — Provision the server

```bash
cd infra/terraform
terraform init
terraform apply -var="domain_name=yourdomain.com"
```

Or use the all-in-one script:

```bash
./infra/scripts/deploy.sh yourdomain.com admin@yourdomain.com
```

### Step 5 — Point your domain to the server

After Terraform runs, it outputs a static IP address. Add an **A Record** in your DNS provider pointing your domain to that IP. SSL certificates will be issued automatically on the first deployment.

---

## 5. GitHub Secrets — Configuration Reference

These values must be added to the GitHub repository under **Settings → Secrets and variables → Actions**.

> ⚠️ Never put these values in code files or share them in plain text. GitHub encrypts them and only exposes them during pipeline runs.

### AWS & Infrastructure

| Secret | Description | Example |
|---|---|---|
| `AWS_ACCESS_KEY_ID` | AWS IAM access key | `AKIAIOSFODNN7EXAMPLE` |
| `AWS_SECRET_ACCESS_KEY` | AWS IAM secret key | `wJalrXUtnFEMI/K7MDENG/...` |
| `DOMAIN_NAME` | Your production domain | `iviss.youragency.gov` |
| `CERTBOT_EMAIL` | Email for SSL certificate alerts | `admin@youragency.gov` |

### Authentication & Security

| Secret | Description | How to generate |
|---|---|---|
| `JWT_SECRET` | Secret key for token signing | `openssl rand -base64 48` |
| `JWT_PRIVATE_KEY_PEM` | RSA private key (single line) | See [Section 9](#9-security--key-generation--rotation) |
| `JWT_PUBLIC_KEY_PEM` | RSA public key (single line) | See [Section 9](#9-security--key-generation--rotation) |
| `ACTIVATION_CODE_PEPPER` | Extra security for OTP codes | `openssl rand -base64 48` |

### Application Configuration

| Secret | Description | Example |
|---|---|---|
| `ENVIRONMENT` | Deployment environment | `production` |
| `LOG_LEVEL` | How much detail to log | `info` |
| `SHIFT_START_HOUR` | Hour agents can start logging in | `6` (for 6:00 AM) |
| `SHIFT_END_HOUR` | Hour agents can no longer log in | `18` (for 6:00 PM) |

### Admin Bootstrap (Initial Admin Account)

These are used to create the first administrator account when the system starts for the first time.

| Secret | Description | Example |
|---|---|---|
| `ADMIN_BOOTSTRAP_EMAIL` | Admin login email | `admin@youragency.gov` |
| `ADMIN_BOOTSTRAP_PASSWORD` | Admin initial password | A strong password |
| `ADMIN_BOOTSTRAP_PHONE` | Admin phone number | `+237600000000` |
| `ADMIN_BOOTSTRAP_USERNAME` | Admin username | `admin` |

### Database

| Secret | Description | Example |
|---|---|---|
| `POSTGRES_USER` | Database username | `iviss_user` |
| `POSTGRES_PASSWORD` | Database password | A strong random password |
| `POSTGRES_DB` | Database name | `iviss_prod` |

### Container Registry

| Secret | Description |
|---|---|
| `REGISTRY_USERNAME` | Your GitHub username |
| `REGISTRY_TOKEN` | GitHub PAT with `read:packages` permission |

### SMS Provider

| Secret | Required when |
|---|---|
| `SMS_PROVIDER` | Always — set to `mock`, `twilio`, `vonage`, or `orange` |
| `VONAGE_API_KEY` | `SMS_PROVIDER=vonage` |
| `VONAGE_API_SECRET` | `SMS_PROVIDER=vonage` |
| `TWILIO_ACCOUNT_SID` | `SMS_PROVIDER=twilio` |
| `TWILIO_AUTH_TOKEN` | `SMS_PROVIDER=twilio` |
| `TWILIO_FROM_NUMBER` | `SMS_PROVIDER=twilio` |
| `ORANGE_CLIENT_ID` | `SMS_PROVIDER=orange` |
| `ORANGE_CLIENT_SECRET` | `SMS_PROVIDER=orange` |
| `ORANGE_SENDER_NUMBER` | `SMS_PROVIDER=orange` |

### Email Provider

| Secret | Required when |
|---|---|
| `EMAIL_PROVIDER` | Always — set to `mock`, `resend`, or `smtp` |
| `RESEND_API_KEY` | `EMAIL_PROVIDER=resend` |
| `RESEND_FROM_EMAIL` | `EMAIL_PROVIDER=resend` |
| `SMTP_HOST` | `EMAIL_PROVIDER=smtp` |
| `SMTP_PORT` | `EMAIL_PROVIDER=smtp` |
| `SMTP_USERNAME` | `EMAIL_PROVIDER=smtp` |
| `SMTP_PASSWORD` | `EMAIL_PROVIDER=smtp` |
| `SMTP_FROM_EMAIL` | `EMAIL_PROVIDER=smtp` |

---

## 6. Provider Configuration (SMS & Email)

IVISS supports multiple providers for sending SMS messages (OTP codes) and emails. You switch between them by changing the `SMS_PROVIDER` and `EMAIL_PROVIDER` secrets.

### SMS Providers

| Provider | Value | Description |
|---|---|---|
| Mock | `mock` | No SMS is sent. The OTP code appears in the server logs. Use this for development and testing. |
| Twilio | `twilio` | International SMS provider. Reliable worldwide coverage. |
| Vonage | `vonage` | International SMS provider (formerly Nexmo). |
| Orange Cameroun | `orange` | Local Cameroonian carrier. Only works with `+237` numbers. |

> **Recommendation for production in Cameroon:** Use `orange` for local numbers or `twilio` for international coverage.

### Email Providers

| Provider | Value | Description |
|---|---|---|
| Mock | `mock` | No email is sent. Content appears in server logs. Use for testing only. |
| Resend | `resend` | Modern email API with high delivery rates. Recommended for production. |
| SMTP | `smtp` | Standard email protocol. Works with Gmail, Outlook, or any custom mail server. |

---

## 7. DNS & SSL (HTTPS)

### Setting up your domain

1. After the server is provisioned, find the static IP address in the AWS Lightsail console or from the Terraform output
2. Log in to your DNS provider (the company where your domain is registered)
3. Add an **A Record**:
   - **Name:** `@` (or your subdomain, e.g. `iviss`)
   - **Value:** the static IP address from step 1
   - **TTL:** 300 (or the lowest available)
4. Wait for DNS to propagate (usually 5–30 minutes)

### SSL Certificate (HTTPS)

SSL is handled automatically using **Let's Encrypt** — a free, trusted certificate authority. On the first deployment after DNS is configured:

- The system requests a certificate for your domain
- Nginx is configured to serve the application over HTTPS
- The certificate renews automatically every 90 days

> ⚠️ The `CERTBOT_EMAIL` secret must be a real, monitored email address. Let's Encrypt sends expiry warnings to this address if automatic renewal fails.

---

## 8. Manual Operations on the Server

If you need to connect to the server directly (for example, to check logs or restart the application), you can SSH into it.

### Viewing live logs

```bash
cd /opt/iviss
docker compose logs -f
```

To see logs for a specific service only:

```bash
docker compose logs -f backend    # Backend API logs
docker compose logs -f frontend   # Frontend/Nginx logs
```

### Restarting the application

```bash
cd /opt/iviss
docker compose down
docker compose up -d
```

### Checking the status of all services

```bash
docker compose ps
```

All services should show status `Up`. If any show `Exit` or `Restarting`, check the logs for that service.

### Pulling the latest version manually

If you need to force an update without going through the CI/CD pipeline:

```bash
cd /opt/iviss
docker compose pull
docker compose up -d
```

---

## 9. Security — Key Generation & Rotation

### Generating a JWT Secret

```bash
openssl rand -base64 48
```

Copy the output and save it as the `JWT_SECRET` GitHub secret.

### Generating an RSA Key Pair (for JWT signing)

```bash
# Step 1 — Generate the private key
openssl genrsa -out jwt-private.pem 2048

# Step 2 — Extract the public key from it
openssl rsa -in jwt-private.pem -pubout -out jwt-public.pem
```

### Formatting keys for GitHub Secrets

GitHub Secrets must be a single line. Use this command to convert the key file to the correct format:

```bash
# For the private key
awk '{printf "%s\\n", $0}' jwt-private.pem

# For the public key
awk '{printf "%s\\n", $0}' jwt-public.pem
```

Copy the entire output (including the `-----BEGIN...` and `-----END...` parts) and paste it as the secret value.

> ⚠️ After rotating keys, all active agent sessions will be invalidated. Agents will need to log in again on their next shift.

### Generating the Activation Code Pepper

```bash
openssl rand -base64 48
```

> ⚠️ If this value is changed after agents have been activated, all existing activation codes become invalid. Only rotate this during a planned maintenance window.

---

## 10. Troubleshooting

### "Unauthorized" or 401 errors on the frontend

**Cause:** The JWT keys in the GitHub Secrets do not match the keys the backend was started with, or a session has expired.

**Fix:** Verify that `JWT_PRIVATE_KEY_PEM` and `JWT_PUBLIC_KEY_PEM` are correctly formatted (single line with `\n` separators) and re-deploy.

---

### Deployment fails with "Conflict: Target already exists" (Terraform)

**Cause:** Terraform is trying to create a resource that already exists on AWS.

**Fix:** Either import the existing resource into Terraform state, or ensure you are using the correct remote state backend.

```bash
cd infra/terraform
terraform import aws_lightsail_instance.iviss <instance-name>
```

---

### Docker images fail to pull on the server

**Cause:** The `REGISTRY_TOKEN` secret has expired or does not have the correct permissions.

**Fix:** Generate a new GitHub Personal Access Token with `read:packages` permission and update the `REGISTRY_TOKEN` secret in GitHub.

---

### SSL certificate not issued / site shows "Not Secure"

**Cause:** DNS has not propagated yet, or the domain does not point to the correct IP.

**Fix:**
1. Verify the A Record is correct using [dnschecker.org](https://dnschecker.org)
2. Wait for propagation (up to 30 minutes)
3. Re-run the deployment once DNS resolves correctly

---

### Application starts but shows a blank page

**Cause:** The frontend container started before the backend was ready, or the OpenAPI client was not generated.

**Fix:**
```bash
cd /opt/iviss
docker compose restart frontend
```

---

### Database connection errors in backend logs

**Cause:** The `POSTGRES_PASSWORD` in the secrets does not match what the database was initialized with, or the database container is not running.

**Fix:**
```bash
cd /opt/iviss
docker compose ps          # Check if db container is running
docker compose logs db     # Check database logs for errors
```

If the password was changed after the database was already initialized, the database volume must be reset (this deletes all data — only do this on a fresh setup):

```bash
docker compose down -v     # ⚠️ Deletes all data
docker compose up -d
```
