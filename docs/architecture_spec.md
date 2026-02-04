# IVISS — Technical Architecture & System Design Documentation

**Version:** 1.0
**Date:** February 2026
**Author:** IVISS Development Team

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [High-Level Architecture](#2-high-level-architecture)
3. [Multi-Tenant Organization Hierarchy](#3-multi-tenant-organization-hierarchy)

## 1. System Overview

**IVISS** is a multi-tenant platform enabling law enforcement and regulatory organizations to:

- Identify vehicles via license plate recognition (manual entry, photo capture, or live scan)
- Query national vehicle databases for registration details
- Verify vehicle compliance status through partner APIs (insurance, customs, technical inspection, wanted status)

**Key Components:**

- **Rust Backend :** Core API server, business logic, database orchestration
- **Android Mobile App:** Field agent interface for vehicle scanning and lookup
- **Web Back-Office :** Administrative interface for supervisors and admins
- **PostgreSQL (Internal):** IVISS-owned data (users, organizations, audit logs)
- **PostgreSQL (External):** National vehicle registry
- **Partner APIs:** Third-party services (insurance, customs, inspection, wanted list)

---

## 2. High-Level Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        A[Android App<br/>Agent Field Device]
        B[Web Back-Office<br/>React SPA]
    end
  
  
    subgraph "Server Infrastructure"
        G[API Gateway<br/>JWT + Rate Limit + CORS]
        W[IVISS WebService<br/>Rust + Axum + Tokio]
    end
  
    subgraph "Data Layer"
        DB1[(PostgreSQL Internal<br/>Organizations, Users,<br/>Agents, Audit Logs)]
        DB2[(PostgreSQL External<br/>National Vehicle DB<br/>Read-Only)]
    end
  
    subgraph "External Partners"
        API1[Insurance API]
        API2[Customs API]
        API3[Inspection API]
        API4[Wanted Vehicles API]
    end
  
    A -->|HTTPS + JWT| G
    B -->|HTTPS + JWT| G
  
    G -->|Authenticated Request| W
    W -->|sqlx Queries<br/>Read-Write| DB1
    W -->|sqlx Queries<br/>Read-Only| DB2
    W -.->|HTTPS + API Key| API1
    W -.->|HTTPS + API Key| API2
    W -.->|HTTPS + API Key| API3
    W -.->|HTTPS + API Key| API4
  
    style A fill:#2dd4a8,stroke:#1a8f6f,color:#000
    style B fill:#60a5fa,stroke:#1e40af,color:#000
    style G fill:#fb923c,stroke:#c2410c,color:#000
    style W fill:#a78bfa,stroke:#6d28d9,color:#000
    style DB1 fill:#f472b6,stroke:#be185d,color:#000
    style DB2 fill:#f87171,stroke:#b91c1c,color:#000
    style API1 fill:#f87171,stroke:#b91c1c,color:#000
    style API2 fill:#f87171,stroke:#b91c1c,color:#000
    style API3 fill:#f87171,stroke:#b91c1c,color:#000
    style API4 fill:#f87171,stroke:#b91c1c,color:#000
```

**Legend:**

- **Green:** Mobile client (Android)
- **Blue:** Web client (React)
- **Orange:** API gateway / router
- **Purple:** Core backend service
- **Pink:** Internal database (IVISS-owned)
- **Red:** External systems (not owned by IVISS)

---

## 3. Multi-Tenant Organization Hierarchy

IVISS supports a hierarchical, multi-tenant organization structure with role-based access control (RBAC).

### 3.1 Organization Hierarchy

```mermaid
graph TD
    SA[Super Admin<br/>System-wide privileges]
  
    SA --> O1[Organization: Police]
    SA --> O2[Organization: Customs]
    SA --> O3[Organization: Inspection Agency]
  
    O1 --> A1[Admin<br/>Manages Police org]
    O2 --> A2[Admin<br/>Manages Customs org]
  
    A1 --> S1[Supervisor<br/>Coordinates agents]
    A1 --> S2[Supervisor<br/>Coordinates agents]
  
    S1 --> AG1[Agent<br/>Field operations]
    S1 --> AG2[Agent<br/>Field operations]
    S2 --> AG3[Agent<br/>Field operations]
  
    A2 --> S3[Supervisor<br/>Coordinates agents]
    S3 --> AG4[Agent<br/>Field operations]
  
    style SA fill:#f87171,stroke:#b91c1c,color:#000
    style O1 fill:#fb923c,stroke:#c2410c,color:#000
    style O2 fill:#fb923c,stroke:#c2410c,color:#000
    style O3 fill:#fb923c,stroke:#c2410c,color:#000
    style A1 fill:#60a5fa,stroke:#1e40af,color:#000
    style A2 fill:#60a5fa,stroke:#1e40af,color:#000
    style S1 fill:#a78bfa,stroke:#6d28d9,color:#000
    style S2 fill:#a78bfa,stroke:#6d28d9,color:#000
    style S3 fill:#a78bfa,stroke:#6d28d9,color:#000
    style AG1 fill:#2dd4a8,stroke:#1a8f6f,color:#000
    style AG2 fill:#2dd4a8,stroke:#1a8f6f,color:#000
    style AG3 fill:#2dd4a8,stroke:#1a8f6f,color:#000
    style AG4 fill:#2dd4a8,stroke:#1a8f6f,color:#000
```

### 3.2 Role Definitions

| Role                  | Scope               | Permissions                                                                                                                                          | Implementation (MVP)                                                                  |
| --------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **Super Admin** | System-wide         | • Create/delete organizations ``• View all organizations``• System configuration``• Access all audit logs                                        | ✅ Implemented `role = "super_admin"` in `members` table                          |
| **Admin**       | Single organization | • Manage members (create/update/delete admins)``• Manage agents (assign/unassign)``• View organization statistics``• Access org-level audit logs | ✅ Implemented `role = "admin"` in `members` table``Scoped to `organization_id` |
| **Supervisor**  | Assigned agents     | • Coordinate field agents ``• View agent activity``• Generate reports``• Handle citizen requests                                                 | ⏭️ Deferred to post-MVP``Will be a separate role or flag                            |
| **Agent**       | Self only           | • Perform vehicle lookups ``• Record controls``• Upload carte grise images``• View own activity history                                          | ✅ Implemented ``Stored in `agents` table``Linked to `managed_by` (admin member)    |
