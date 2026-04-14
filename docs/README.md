# IVISS Documentation

Welcome to the IVISS documentation. Start here to understand the system.

## 🚀 For Deployment/DevOps Team

**New to deployment? Start here:**

- [Deployment Index](DEPLOYMENT_INDEX.md) - Complete deployment documentation guide
- [Deployment Overview](DEPLOYMENT_OVERVIEW.md) ⭐ **START HERE** - Current state and gaps
- [CI/CD Pipelines](CICD_PIPELINES.md) - Automation and workflows
- [Container Architecture](CONTAINER_ARCHITECTURE.md) - Docker services and configuration
- [Infrastructure & Hosting](INFRASTRUCTURE_AND_HOSTING.md) - Infrastructure requirements

## Getting Started

- [Overview](overview.md) - What IVISS does and who uses it
- [Docker Setup](docker_setup.md) - How to run the application locally

## Understanding the System

- [Architecture](architecture_spec.md) - Technical architecture and how components work together
- [Components](components.md) - Detailed guide to backend and frontend components
- [Database Schema](schema_simple.md) - Simple explanation of database tables
- [Database Schema (Detailed)](schema.md) - Complete schema with business context and recommendations

## Authentication & Security

- [Auth Tokens](auth_tokens.md) - How JWT tokens work
- [Daily Login Flow](daily_opertational_login_flow.md) - Agent shift-based authentication
- [Auto Refresh](auto_refresh_signature.md) - Token refresh mechanism
- [Session Management](admin_session_management.md) - Admin session controls
- [Session Termination](admin_session_termination.md) - How admins terminate sessions

## Frontend

- [Frontend Architecture](FE_ARCHITECTURE.md) - React app structure and patterns
- [Admin RBAC](fe_admin_rbac.md) - Role-based access control in the frontend
- [UI Design](ui-design.md) - Design system and components

## Operations

- [Monitoring](monitoring.md) - Prometheus metrics and Grafana dashboards
- [User Registration](User_registration.md) - How users are created and activated

## Development

- [Manual RBAC Testing](manual_rbac_testing.md) - Testing role-based access
- [Sequence Diagrams & Flows](sequence_diagrams_&_flows.md) - Visual flow diagrams
- [Data Models](data.md) - TypeScript interfaces and constants

## Quick Reference

### What is IVISS?

A platform for law enforcement to check vehicles during roadside inspections. Agents scan license plates, the system checks compliance (insurance, technical inspection, stolen status), and everything is logged with GPS location.

### Who uses it?

- Field agents (mobile app with OTP login)
- Managers (back-office dashboard)
- Administrators (full system access)

### Tech stack

- Backend: Rust + Axum + PostgreSQL + Redis
- Frontend: React + TypeScript + Vite
- All services run in Docker

### Key features

- License plate OCR
- Vehicle compliance checks
- Multi-tenant with data isolation
- Role-based access control
- Audit logging
- GPS-tagged control records
