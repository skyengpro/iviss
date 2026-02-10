# IVISS - Intelligent Vehicle Identification & Security System

## Project Overview

IVISS is a multi-tenant platform for law enforcement and regulatory organizations to identify vehicles, perform compliance checks, and manage field operations.

**Architecture:**
- **Frontend**: React + TypeScript + Vite + shadcn/ui (Mobile-first design)
- **Backend**: Rust + Axum + PostgreSQL (Dockerized with live reloading)
- **Database**: PostgreSQL 9.4 (Dockerized)

## Features

- **License Plate Recognition**: Real-time OCR scanning of vehicle plates
- **Control History**: Tracking and management of vehicle controls
- **Alert System**: Instant notification for flagged vehicles
- **Mobile First**: Optimized for field agents
- **Multi-Tenant**: Role-based access control (Super Admin, Admin, Supervisor, Agent)

---

## Technologies

### Frontend
- React
- TypeScript
- Vite
- shadcn/ui (Radix primitives)
- Tailwind CSS
- React Query
- React Router DOM
- Tesseract.js (OCR)
- react-webcam

### Backend
- Rust
- Axum (Web framework)
- Tokio (Async runtime)
- SQLx (Database)
- Tower-HTTP (Middleware)

### Database
- PostgreSQL 9.4 (via Docker)

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
