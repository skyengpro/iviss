# IVISS

## Table of Contents

- [Project Overview](#project-overview)
- [Features](#features)
- [Architecture](#architecture)
- [Technologies](#technologies)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Clone and Setup](#clone-and-setup)
  - [Start Backend Services](#start-backend-services)
  - [Verify Backend is Running](#verify-backend-is-running)
  - [Start Frontend (Optional)](#start-frontend-optional)
- [Testing the Setup](#testing-the-setup)
  - [Test Backend Locally](#test-backend-locally)
  - [Test from Mobile Device](#test-from-mobile-device)
- [Development Features](#development-features)
  - [Live Reloading](#live-reloading)
  - [Database Access](#database-access)
  - [Data Persistence](#data-persistence)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

## Project Overview

IVISS (Intelligent Vehicle Identification & Security System) is a platform that helps law enforcement and regulatory agencies check vehicles during roadside inspections. Field agents use mobile devices to scan license plates and check compliance status. Back-office staff manage users, organizations, and review submitted documents.

See [docs/overview.md](docs/overview.md) for a complete system overview.

## Features

- **License Plate Recognition**: Real-time Optical Character Recognition (OCR) scanning of vehicle license plates for rapid identification and data retrieval.
- **Control History**: Comprehensive tracking and management of all vehicle control operations, including details of inspections, violations, and resolutions.
- **Alert System**: Instant notification system for flagged vehicles (e.g., stolen, unregistered, or vehicles with outstanding warrants), enabling immediate action by field agents.
- **Mobile First**: User interface and experience are optimized for mobile devices, ensuring field agents can efficiently perform tasks on the go.
- **Progressive Web App (PWA)**: Installable on Android, iOS, and Desktop with offline support and automatic updates. [Learn more →](./PWA_IMPLEMENTATION_SUMMARY.md)
- **Multi-Tenant**: Supports multiple organizations with isolated data and configurations.
- **Role-Based Access Control**: Granular access control system with predefined roles (Super Admin, Admin, Supervisor, Agent) to manage permissions and data visibility.

## Architecture

- Frontend: React + TypeScript + Vite + shadcn/ui
- Backend: Rust + Axum
- Database: PostgreSQL 15
- Cache: Redis 7
- All services run in Docker containers

See [docs/architecture_spec.md](docs/architecture_spec.md) and [docs/components.md](docs/components.md) for technical details.

---

## Technologies

### Frontend

- React: A JavaScript library for building user interfaces.
- TypeScript: A typed superset of JavaScript that compiles to plain JavaScript.
- Vite: A fast build tool for modern web projects.
- shadcn/ui (Radix primitives): A collection of re-usable components built with Radix UI and Tailwind CSS.
- Tailwind CSS: A utility-first CSS framework for rapidly building custom designs.
- React Query: Powerful asynchronous state management for React.
- React Router DOM: Declarative routing for React.
- Tesseract.js (OCR): JavaScript library for performing OCR.
- react-webcam: React component for accessing and displaying webcam streams.

### Backend

- Rust: A systems programming language focused on safety, speed, and concurrency.
- Axum (Web framework): A web application framework built with Tokio, Tower, and Hyper.
- Tokio (Async runtime): An asynchronous runtime for Rust.
- SQLx (Database): An asynchronous, pure Rust SQL crate.
- Tower-HTTP (Middleware): A collection of HTTP middleware for Tower.

### Database

- PostgreSQL 9.4 (via Docker): A powerful, open-source object-relational database system.

---

## Quick Start

### Prerequisites

- **Docker** (20.10+) and **Docker Compose** (2.0+) - [Installation Guide](https://docs.docker.com/get-docker/)
- **Node.js** (16+) and **npm** - Only for frontend development

### 1. Clone and Setup

```bash
# Clone repo & copy environment template
cp .env.example .env

# Set real local secrets before starting the stack
# At minimum: POSTGRES_PASSWORD and EXTERNAL_POSTGRES_PASSWORD
```

### 2. Start Backend Services

```bash
# Start the local development stack
docker compose up -d

# Start the production-like local stack
docker compose --profile prod up -d db redis backend-prod frontend-prod metrics

# View real-time logs
docker compose logs -f backend
docker compose logs -f db

# Stop services (keeps data)
docker compose down

# Stop and remove all data
docker compose down -v
```

### 3. Verify Backend is Running

```bash
# Test health endpoint
curl http://localhost:3000/api/v1/health
# Should return: OK

# Check service status
docker compose ps
# Both iviss-backend and iviss-db should show "Up"
```

### 4. Start Frontend (Optional)

```bash
cd frontend
npm install
npm run dev
# Access at http://localhost:5173
```

---

## Testing the Setup

### Test Backend Locally

```bash
# Health check
curl http://localhost:3000/api/v1/health

# Get your local IP (for mobile testing)
hostname -I | awk '{print $1}'
```

### Test from Mobile Device

**Requirements:**

- Mobile device on the **same WiFi network** as development machine
- Backend services running (`docker compose ps` shows both services Up)

**Steps:**

1. **Get your local IP address:**

   ```bash
   hostname -I | awk '{print $1}'
   # Example output: 192.168.1.233
   ```

2. **Test from mobile browser:**

   ```
   http://YOUR_IP:3000/api/v1/health
   ```

   You should see "OK" response.

3. **Configure frontend for mobile:**

   Update `frontend/.env`:

   ```env
   VITE_API_URL=http://YOUR_IP:3000
   ```

   Then rebuild frontend:

   ```bash
   cd frontend
   npm run build
   ```

**Expected Results:**

- ✅ Mobile browser can access `http://YOUR_IP:3000/api/v1/health`
- ✅ Backend logs show incoming requests on the health endpoint
- ❌ **If you get connection timeout:** Check firewall or ensure devices are on same network

---

## Development Features

### Live Reloading

The backend uses **cargo-watch** for automatic recompilation on code changes:

1. Start services: `docker compose up -d`
2. Edit any Rust file in `iviss-backend/src/`
3. Watch logs: `docker compose logs -f backend`
4. Changes automatically trigger recompilation and restart

**Example:**

```bash
# In one terminal
docker compose logs -f backend

# In another terminal
echo "// test change" >> iviss-backend/src/main.rs

# First terminal will show:
# [Running 'cargo run']
# Compiling iviss-backend...
# INFO Listening on 0.0.0.0:3000
```

### Database Access

```bash
# Connect to database
docker compose exec db psql -U iviss_user -d iviss_db
```

### Data Persistence

Database data is stored in a Docker volume and persists across container restarts:

```bash
# Data survives this:
docker compose down
docker compose up -d

# This DELETES all data:
docker compose down -v
```

### Dev vs Prod Local

- `docker compose up -d` starts the development services: hot reload for backend and frontend, source mounts, Adminer.
- `docker compose --profile prod up -d db redis backend-prod frontend-prod metrics` starts the production-like services: release backend image and nginx frontend image, without dev mounts.

---

## Project Structure

```
iviss/
├── .github/                  # GitHub Actions workflows
├── docs/                     # Project documentation (architecture, data, schema, diagrams)
├── frontend/                 # Frontend application (React, TypeScript)
├── iviss-backend/            # Backend application (Rust, Axum)
├── infra/                    # Infrastructure-as-Code (Terraform, Ansible)
```

---

## Production Infrastructure

[![Deployment Status](https://github.com/skyengpro/iviss/actions/workflows/deploy-aws.yml/badge.svg)](https://github.com/skyengpro/iviss/actions/workflows/deploy-aws.yml)

**Production URL:** [https://iviss-prod.vpn.kivoyo.com/](https://iviss-prod.vpn.kivoyo.com/)

The platform is deployed using **Infrastructure-as-Code (Terraform)** and **Automated CI/CD (GitHub Actions)** on **AWS Lightsail**.

### Infrastructure Details

- **Hosting:** AWS Lightsail (eu-west-1)
- **Instance:** 2 vCPUs, 2 GB RAM, 60 GB SSD (Bundle: `small_3_0`)
- **Cost:** ~$15-20/month
- **Orchestration:** Docker Compose
- **Configuration:** Ansible
- **State Management:** S3 + DynamoDB
- **SSL/TLS:** Let's Encrypt (automatic renewal)
- **Reverse Proxy:** Nginx

### Deployment Architecture

```
GitHub Push (main branch)
         ↓
GitHub Actions (CI/CD)
         ↓
Docker Images → GHCR
         ↓
Terraform (Infrastructure)
         ↓
Ansible (Configuration)
         ↓
AWS Lightsail (Production)
```

### Deployment Status

- ✅ Automated deployment pipeline
- ✅ Infrastructure as Code (Terraform)
- ✅ Configuration management (Ansible)
- ✅ Secrets management (GitHub Secrets)
- ✅ SSL/TLS certificates (Let's Encrypt)
- ✅ Container orchestration (Docker Compose)
- ✅ Health monitoring
- ⚠️ Database backups (manual)
- ⚠️ Production monitoring (planned)

### Quick Links

- **Production App:** [https://iviss-prod.vpn.kivoyo.com/](https://iviss-prod.vpn.kivoyo.com/)
- **API Health:** [https://iviss-prod.vpn.kivoyo.com/api/v1/health](https://iviss-prod.vpn.kivoyo.com/api/v1/health)
- **Deployment Docs:** [docs/DEPLOYMENT_CURRENT_STATE.md](docs/DEPLOYMENT_CURRENT_STATE.md)
- **GitHub Actions:** [View Workflows](https://github.com/skyengpro/iviss/actions)

---

## Documentation

### For Developers

- [Overview](docs/overview.md) - What IVISS does and who uses it
- [Architecture](docs/architecture_spec.md) - Technical architecture
- [Components](docs/components.md) - Backend and frontend components
- [Database Schema](docs/schema.md) - Complete database documentation
- [Docker Setup](docs/docker_setup.md) - Local development setup

### For DevOps/Deployment

- [**Deployment Current State**](docs/DEPLOYMENT_CURRENT_STATE.md) ⭐ **START HERE** - Complete deployment guide
- [Deployment Index](docs/DEPLOYMENT_INDEX.md) - Navigation hub for all deployment docs
- [CI/CD Pipelines](docs/CICD_PIPELINES.md) - GitHub Actions workflows
- [Container Architecture](docs/CONTAINER_ARCHITECTURE.md) - Docker services and configuration
- [Infrastructure & Hosting](docs/INFRASTRUCTURE_AND_HOSTING.md) - Infrastructure requirements

### For Operations

- [Monitoring](docs/monitoring.md) - Metrics and dashboards
- [Auth Tokens](docs/auth_tokens.md) - JWT authentication
- [Daily Login Flow](docs/daily_opertational_login_flow.md) - Agent authentication

---

## Operational Documentation
- [**Deployment & Infrastructure Guide**](docs/DEPLOYMENT_CURRENT_STATE.md): The definitive guide for production.

---

## Contributing

For internal contributions, please coordinate with the project lead. All changes should be made on dedicated feature branches and reviewed before merging into the main development branch.

## License

This project is proprietary and all rights are reserved.
