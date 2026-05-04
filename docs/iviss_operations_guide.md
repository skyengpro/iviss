# IVISS — Deployment & Release Overview

---

## 1. Hosting Infrastructure

IVISS is hosted on **AWS Lightsail** — a managed cloud server provided by Amazon Web Services. The application runs as isolated containers, making it portable, secure, and easy to update.

| Component | Details |
|---|---|
| Cloud provider | AWS Lightsail |
| Server | 2 vCPUs, 2 GB RAM, 60 GB SSD |
| Operating system | Ubuntu 22.04 LTS |
| Backend API | Containerised Rust application |
| Frontend | Containerised React application served via Nginx |
| Database | PostgreSQL with persistent storage |

The entire server infrastructure is defined as code — meaning the environment can be reproduced or migrated reliably without manual configuration.

---

## 2. Deployment Strategy

Every update to the application goes through a fully automated pipeline. Once a change is validated and approved by the development team, it is deployed to the server without any manual steps.

The pipeline follows this sequence:

1. **Code review** — changes are reviewed by the team before being accepted
2. **Automated checks** — tests and quality checks run automatically
3. **Release creation** — a new version number is assigned and published
4. **Packaging** — the application is packaged into Docker images and stored in a private registry
5. **Deployment** — the server pulls the new images and restarts with the updated version

This approach ensures that only reviewed, tested code reaches the production server.

---

## 3. Versioning

Each deployment produces a versioned release following the **Semantic Versioning** standard — a widely adopted convention in the software industry. Version numbers take the form `MAJOR.MINOR.PATCH` (for example `v1.2.3`).

- A **patch** release (e.g. `v0.1.1`) means a bug was fixed
- A **minor** release (e.g. `v0.2.0`) means new functionality was added
- A **major** release (e.g. `v1.0.0`) means a significant change was made to the system

Version numbers are assigned automatically based on the nature of the changes — no manual decision is required. All releases are published on the project's GitHub page with a full list of changes included.

---

## 4. Security & Configuration

All sensitive configuration — including database credentials, authentication keys, and third-party API keys — is stored securely as encrypted secrets in the CI/CD system. These values are never stored in the codebase and are only injected into the server at deployment time.

The platform supports multiple providers for SMS and email notifications, configurable without code changes:

- **SMS:** Orange Cameroun, Twilio, or Vonage
- **Email:** SMTP (Gmail, Outlook, or custom) or Resend

---

*IVISS — Internal Technical Documentation*
