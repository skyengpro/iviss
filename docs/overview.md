# IVISS — Project Overview

IVISS (Intelligent Vehicle Identification & Security System) is a platform that helps law enforcement and regulatory agencies check vehicles during roadside inspections.

## What It Does

Field agents use their mobile devices to scan or type in license plates. The system checks if the vehicle has valid insurance, passed technical inspection, or is reported stolen. Every check is logged with GPS location and timestamp. Back-office administrators manage users, organizations, and review vehicle documents that agents submit from the field.

## Who Uses It

- Field Agents: Use mobile web app with device activation and daily OTP codes
- Managers: Use back-office web app with email and password
- Administrators: Use back-office web app with email and password

## System Components

- Backend: Rust + Axum (REST API, authentication, business logic)
- Frontend: React + TypeScript + Vite (mobile and back-office interface)
- Database: PostgreSQL 15 (all application data)
- Cache: Redis 7 (OTP codes, session data, rate limits)
- SMS: Twilio (OTP delivery, mocked in development)
- Monitoring: Prometheus + Grafana (frontend metrics)

## How It Works

The frontend (mobile and back-office) talks to the backend API using HTTPS and JWT tokens. The backend handles authentication, queries the PostgreSQL database for vehicle information, and uses Redis for temporary data like OTP codes. All middleware (CORS, auth, rate limiting) runs inside the backend service.

## Multi-Tenant Setup

Each organization (police brigade, customs office, border control) has its own isolated data. Agents and managers can only see data from their organization. Administrators have access to all organizations.

## What's Implemented

✅ Device activation and daily OTP login for agents
✅ Email/password login for admins and managers
✅ JWT tokens (15-minute access tokens, 30-day refresh tokens)
✅ Vehicle search by plate number
✅ Server-side OCR for license plates
✅ Control record logging with GPS
✅ User and organization management
✅ Session management (admin can terminate agent sessions)
✅ Audit log with CSV export
✅ Dashboard statistics
✅ Shift hours enforcement
✅ Frontend metrics collection

❌ External APIs (insurance, customs) - currently using internal database only
❌ Native Android app - using responsive web app instead
⚠️ Gray card approval workflow - backend done, frontend UI incomplete
⚠️ Organization admin role - database ready, logic not implemented

## Getting Started

```bash
# Copy environment file
cp iviss-backend/.env.example iviss-backend/.env

# Edit .env and set: POSTGRES_PASSWORD, JWT keys, ADMIN_BOOTSTRAP_* variables

# Start everything
docker compose up -d
```

Access the application:
- Backend API: http://localhost:3000
- Frontend: http://localhost:8080
- API Documentation: http://localhost:3000/docs
- Database Admin: http://localhost:8081

See [docker_setup.md](docker_setup.md) for detailed setup instructions.
