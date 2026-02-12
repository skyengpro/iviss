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

IVISS (Intelligent Vehicle Identification & Security System) is a robust, multi-tenant platform designed to empower law enforcement and regulatory organizations. It streamlines vehicle identification, automates compliance checks, and provides comprehensive tools for managing field operations efficiently. The system aims to enhance public safety and regulatory adherence through advanced technology.

**Target Users:** Law enforcement agencies, government regulatory bodies, and organizations responsible for vehicle compliance and security.

## Features

- **License Plate Recognition**: Real-time Optical Character Recognition (OCR) scanning of vehicle license plates for rapid identification and data retrieval.
- **Control History**: Comprehensive tracking and management of all vehicle control operations, including details of inspections, violations, and resolutions.
- **Alert System**: Instant notification system for flagged vehicles (e.g., stolen, unregistered, or vehicles with outstanding warrants), enabling immediate action by field agents.
- **Mobile First**: User interface and experience are optimized for mobile devices, ensuring field agents can efficiently perform tasks on the go.
- **Multi-Tenant**: Supports multiple organizations with isolated data and configurations.
- **Role-Based Access Control**: Granular access control system with predefined roles (Super Admin, Admin, Supervisor, Agent) to manage permissions and data visibility.

## Architecture:

- **Frontend**: React + TypeScript + Vite + shadcn/ui (Mobile-first design)
- **Backend**: Rust + Axum + PostgreSQL (Dockerized with live reloading)
- **Database**: PostgreSQL 9.4 (Dockerized)

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

# Edit .env if needed (defaults work for local development)
```

### 2. Start Backend Services

```bash
# Start PostgreSQL database + Rust backend
docker compose up -d

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
curl http://localhost:3000/health
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
curl http://localhost:3000/health

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
   http://YOUR_IP:3000/health
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
- ✅ Mobile browser can access `http://YOUR_IP:3000/health`
- ✅ Backend logs show incoming requests: `INFO Request: GET /health`
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

---

## Project Structure

```
iviss/
├── .github/                  # GitHub Actions workflows
├── docs/                     # Project documentation (architecture, data, schema, diagrams)
├── frontend/                 # Frontend application (React, TypeScript)
│   ├── public/               # Static assets
│   ├── src/                  # Frontend source code
│   │   ├── components/       # Reusable UI components
│   │   ├── contexts/         # React Context API for global state
│   │   ├── hooks/            # Custom React hooks
│   │   ├── i18n/             # Internationalization files
│   │   ├── lib/              # Utility functions and libraries
│   │   ├── pages/            # Application pages (mobile and backoffice)
│   │   ├── router/           # React Router configuration
│   │   └── services/         # API service integrations (mocked for now)
│   └── ...                   # Other frontend configuration files
└── iviss-backend/            # Backend application (Rust, Axum)
    ├── .cargo/               # Cargo configuration
    ├── docs/                 # Backend-specific documentation
    ├── migrations/           # Database migration scripts
    ├── scripts/              # Utility scripts (e.g., database initialization)
    └── src/                  # Backend source code
        ├── db/               # Database connection and query logic
        ├── middleware/       # Custom Axum middleware
        └── ...               # Other backend modules (config, errors, main, routes)
```

---

## Contributing

For internal contributions, please coordinate with the project lead. All changes should be made on dedicated feature branches and reviewed before merging into the main development branch.

## License

This project is proprietary and all rights are reserved.
