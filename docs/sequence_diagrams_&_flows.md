# IVISS — Sequence diagrams & communication flows Documentation

**Version:** 1.0
**Date:** February 2026
**Author:** IVISS Development Team

---

## Table of Contents

1. [Partner API Integration Flow](#5-Partner API Integration Flow)

2. [Sequence Diagrams](#6-sequence-diagrams)

3. [Client-Server Communication](#7-client-server-communication)

### 1. Partner API Integration Flow

```mermaid
sequenceDiagram
    participant A as Android App
    participant W as IVISS WebService
    participant E as External Vehicle DB
    participant I as Insurance API
    participant C as Customs API
    participant T as Inspection API
    participant V as Wanted API
  
    A->>W: GET /vehicles/lookup?plate=AB-123-CD
    Note over W: Validate JWT
    Note over W: Validate plate format
  
    W->>E: SELECT * FROM vehicles WHERE plate = 'AB-123-CD'
    E-->>W: Vehicle data (chassis: VF1RFD...)
  
    par Parallel API Calls
        W->>I: GET /vehicles/{chassis}/insurance
        I-->>W: { valid: true, expiry: "2026-08-20" }
    and
        W->>C: GET /vehicles/{chassis}/customs
        C-->>W: { cleared: true, date: "2019-03-10" }
    and
        W->>T: GET /vehicles/{chassis}/inspection
        T-->>W: { valid: true, next_due: "2027-06-12" }
    and
        W->>V: GET /vehicles/{chassis}/wanted
        V-->>W: { is_wanted: false }
    end
  
    Note over W: Aggregate all statuses
    W-->>A: 200 OK + Vehicle + Statuses JSON
```

**Error Handling:**

- **Timeout (3s):** If any partner API exceeds 3 seconds, IVISS returns a partial response with `"status": "unknown"` for that partner.
- **Service Unavailable:** If a partner API returns 503, IVISS logs the error and marks that status as `"unavailable"`.
- **Rate Limiting:** Partner APIs may enforce rate limits. IVISS caches responses for 5 minutes to reduce duplicate calls.


## 2. Sequence Diagrams

### 2.1 Agent Login & Vehicle Lookup (Full Flow)

```mermaid
sequenceDiagram
    participant Agent as Field Agent
    participant App as Android App
    participant Router as Router/Firewall
    participant GW as API Gateway
    participant WS as WebService
    participant DB as PostgreSQL (Internal)
    participant ExtDB as PostgreSQL (External)
    participant Partners as Partner APIs
  
    %% Login Flow
    Agent->>App: Enter email & password
    App->>Router: POST /auth/login (HTTPS)
    Router->>GW: Forward to API Gateway
    GW->>WS: Forward authenticated request
    WS->>DB: SELECT * FROM members WHERE email = ?
    DB-->>WS: Member record (with password_hash)
    Note over WS: Verify password (argon2)
    Note over WS: Generate JWT tokens
    WS->>DB: INSERT refresh_token (hashed)
    WS-->>GW: 200 + { access_token, refresh_token }
    GW-->>Router: Response
    Router-->>App: Response
    App-->>Agent: Login successful
  
    %% Vehicle Lookup Flow
    Agent->>App: Scan plate or enter "AB-123-CD"
    App->>Router: GET /vehicles/lookup?plate=AB-123-CD<br/>(Authorization: Bearer {token})
    Router->>GW: Forward
    Note over GW: Extract & verify JWT
    GW->>WS: Authenticated request + user claims
    Note over WS: Validate plate format (regex)
    WS->>ExtDB: SELECT * FROM vehicles WHERE plate = 'AB-123-CD'
    ExtDB-->>WS: Vehicle data (chassis, owner, etc.)
  
    par Parallel Partner Calls
        WS->>Partners: GET /insurance/{chassis}
        Partners-->>WS: Insurance status
    and
        WS->>Partners: GET /customs/{chassis}
        Partners-->>WS: Customs status
    and
        WS->>Partners: GET /inspection/{chassis}
        Partners-->>WS: Inspection status
    and
        WS->>Partners: GET /wanted/{chassis}
        Partners-->>WS: Wanted status
    end
  
    Note over WS: Aggregate all statuses
    WS->>DB: INSERT control_log (audit)
    WS-->>GW: 200 + Vehicle + Statuses JSON
    GW-->>Router: Response
    Router-->>App: Response
    App-->>Agent: Display vehicle info + statuses
```

---

### 2.2 Admin Creates Member (Hierarchy Enforcement)

```mermaid
sequenceDiagram
    participant Admin as Organization Admin
    participant Web as Web Back-Office
    participant Router as Router/Firewall
    participant GW as API Gateway
    participant WS as WebService
    participant DB as PostgreSQL (Internal)
  
    Admin->>Web: Fill form: email, password, role
    Web->>Router: POST /organizations/{org_id}/members<br/>(Authorization: Bearer {admin_token})
    Router->>GW: Forward
    Note over GW: Extract & verify JWT
    Note over GW: Extract admin's role from token claims
    GW->>WS: Authenticated request + claims
  
    Note over WS: Assert caller.organization_id == org_id
    Note over WS: Assert caller.role == "admin" or "super_admin"
  
    alt Authorization Failed
        WS-->>GW: 403 Forbidden
        GW-->>Router: 403
        Router-->>Web: 403
        Web-->>Admin: Error: Not authorized
    else Authorization Success
        Note over WS: Hash password (argon2, spawn_blocking)
        WS->>DB: INSERT INTO members (org_id, email, hash, role)
        DB-->>WS: New member record (no password_hash in response)
        WS-->>GW: 201 Created + Member JSON
        GW-->>Router: 201
        Router-->>Web: 201
        Web-->>Admin: Success: Member created
    end
```

---

### 2.3 Member Assigns Agent (Ownership Check)

```mermaid
sequenceDiagram
    participant Member as Admin Member
    participant Web as Web Back-Office
    participant Router as Router/Firewall
    participant GW as API Gateway
    participant WS as WebService
    participant DB as PostgreSQL (Internal)
  
    Member->>Web: Fill form: first_name, last_name, phone_imei
    Web->>Router: POST /organizations/{org_id}/members/{member_id}/agents<br/>(Authorization: Bearer {token})
    Router->>GW: Forward
    Note over GW: Extract & verify JWT
    Note over GW: Extract member_id from token claims
    GW->>WS: Authenticated request + claims
  
    Note over WS: Assert caller.id == member_id (ownership)
  
    alt Ownership Failed
        WS-->>GW: 403 Forbidden
        GW-->>Router: 403
        Router-->>Web: 403
        Web-->>Member: Error: Cannot manage other member's agents
    else Ownership Success
        Note over WS: Check phone_imei uniqueness
        WS->>DB: SELECT COUNT(*) FROM agents WHERE phone_imei = ?
        DB-->>WS: count = 0
  
        alt IMEI Duplicate
            WS-->>GW: 409 Conflict
            GW-->>Router: 409
            Router-->>Web: 409
            Web-->>Member: Error: IMEI already registered
        else IMEI Unique
            WS->>DB: INSERT INTO agents (org_id, managed_by, ...)
            DB-->>WS: New agent record
            WS-->>GW: 201 Created + Agent JSON
            GW-->>Router: 201
            Router-->>Web: 201
            Web-->>Member: Success: Agent assigned
        end
    end
```

## 3. Client-Server Communication

### 3.1 Network Topology

```mermaid
graph LR
    subgraph "Client Devices (Internet)"
        A[Android App<br/>Agent Phone<br/>IP: Dynamic]
        B[Web Browser<br/>Admin Desktop<br/>IP: Dynamic]
    end
  
    subgraph "Public Network"
        R[Router/Firewall<br/>Public IP: X.X.X.X<br/>Port: 443 HTTPS]
    end
  
    subgraph "Private Network (IVISS Infrastructure)"
        GW[API Gateway<br/>Internal IP: 192.168.1.10<br/>Port: 8000]
        WS[WebService<br/>Internal IP: 192.168.1.11<br/>Port: 8000]
        DB1[(PostgreSQL Internal<br/>192.168.1.20:5432)]
        DB2[(PostgreSQL External<br/>192.168.1.21:5433)]
    end
  
    A -->|HTTPS :443| R
    B -->|HTTPS :443| R
    R -->|Port Forward<br/>443 -> 8000| GW
    GW -->|Internal HTTP| WS
    WS --> DB1
    WS --> DB2
  
    style R fill:#fb923c,stroke:#c2410c,color:#000
    style GW fill:#fb923c,stroke:#c2410c,color:#000
    style WS fill:#a78bfa,stroke:#6d28d9,color:#000
```
