# BE-08 - Daily Login Flow (Agent)

**Ticket:** BE-08
**Last updated:** 2026-05-03

---

## What is this?

Every day, before going on patrol, an agent confirms their identity with their
badge ID, registered device, and a 6-digit OTP sent by SMS. A successful login
opens the shift and gives the agent access until the configured organization
work window ends.

The backend no longer uses Redis for this flow. Temporary data is stored in an
in-process Moka cache (`AppCache`) and persistent session/auth data remains in
PostgreSQL.

---

## Actors

| Actor          | Role |
| -------------- | ---- |
| **AgentApp**   | Android app on the agent's registered phone |
| **Backend**    | IVISS server and auth handlers |
| **Moka Cache** | In-memory `AppCache`: OTPs, OTP rate limits, refresh nonces, cached org work times, short-lived JTI blacklist cache |
| **PostgreSQL** | Main database: users, organizations, devices, refresh tokens, access token blacklist |
| **SMS**        | Configured SMS provider that sends the OTP |
| **Admin**      | Back-office user who can terminate sessions and manage accounts/devices |

---

## Cache Data

| Cache | Key | Value | TTL / behavior |
| ----- | --- | ----- | -------------- |
| `otp_store` | `user_id` | OTP hash, attempt count, absolute expiry | 5 minutes; deleted after successful validation |
| `rate_limit` | phone number | OTP request count | 10 minutes; max 3 requests |
| `refresh_nonce` | `device_id` | nonce to be signed by the device | 60 seconds; consumed on verification |
| `jti_blacklist` | access-token JTI | empty marker | 3 minutes; also backed by PostgreSQL for persistence |
| `org_work_time` | `organization_id` | `(start_work_time, end_work_time)` in minutes since midnight | no TTL; loaded at startup and filled on cache miss |

---

## Device Status

The database enum currently supports these device states:

| Status | Meaning in the daily login flow |
| ------ | ------------------------------- |
| `INACTIVE` | Registered device is allowed to request and verify daily login. |
| `ACTIVE` | Device is working in the current shift and can refresh access tokens. |
| `SUSPENDED` | Device is blocked. OTP request and verification are rejected. |
| `REVOKED` | Legacy/blocked status. Verification rejects it. |
| `PENDING` | Legacy registration status; not used as the normal daily-login standby state. |

Only successful OTP verification sets the registered device to `ACTIVE` and
stores today's `shift_start` and `shift_end` in `devices.metadata`.

---

## Flow 1 - Request Daily OTP

The agent enters their badge ID from a registered device. The backend verifies
that the user is an agent, checks the organization's configured work window
using the Moka organization cache, validates the device, then stores the OTP in
Moka and sends it by SMS.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant Moka as Moka AppCache
    participant PostgreSQL
    participant SMS

    AgentApp->>Backend: POST /auth/request-daily-login (badgeId, deviceId)
    Backend->>PostgreSQL: Find user by badgeId
    PostgreSQL-->>Backend: user(id, role, status, phone_number)

    Backend->>Backend: Require role = agent and status != SUSPENDED
    Backend->>PostgreSQL: Load user's organization_id
    PostgreSQL-->>Backend: organization_id

    Backend->>Moka: Get org_work_time[organization_id]
    alt cache hit
        Moka-->>Backend: start/end work minutes
    else cache miss
        Backend->>PostgreSQL: SELECT start_work_time, end_work_time
        PostgreSQL-->>Backend: start/end work minutes
        Backend->>Moka: Cache org_work_time[organization_id]
    end

    Backend->>Backend: Check current UTC+1 local time is inside work window
    Backend->>PostgreSQL: Find device by deviceId + userId
    PostgreSQL-->>Backend: device(status, revoked_at)
    Backend->>Backend: Reject if device is missing, SUSPENDED, or terminated today

    Backend->>Moka: Read rate_limit[phone_number]
    Moka-->>Backend: request count or empty
    Backend->>Backend: Reject if count >= 3 in 10 minutes
    Backend->>Moka: Increment rate_limit[phone_number]

    Backend->>Backend: Generate 6-digit OTP and HMAC-SHA256 hash it
    Backend->>Moka: Store otp_store[user_id] = hash + attempts=0 + expires_at=5min
    Backend->>SMS: Send OTP to user's phone number
    SMS-->>AgentApp: OTP SMS
    Backend-->>AgentApp: 201 OTP sent successfully
```

---

## Flow 2 - Verify Daily Login

The agent submits the OTP. The backend consumes the cached OTP, computes today's
shift bounds from the organization work window, issues an access token, activates
the device, and creates a refresh token only if the device does not already have
a valid one.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant Moka as Moka AppCache
    participant PostgreSQL

    AgentApp->>Backend: POST /auth/verify-daily-login (badgeId, activationCode, deviceId)
    Backend->>PostgreSQL: Find user by badgeId and left join device by deviceId
    PostgreSQL-->>Backend: user_id, user_role, user_status, device_status

    Backend->>Backend: Require role=agent, user_status=ACTIVE
    Backend->>Backend: Reject if device_status is SUSPENDED or REVOKED

    Backend->>Moka: Get otp_store[user_id]
    alt OTP missing or expired
        Moka-->>Backend: empty
        Backend-->>AgentApp: 401 OTP expired or not found
    else OTP present
        Moka-->>Backend: OTP hash + attempts + expires_at
        Backend->>Backend: Validate 6-digit format and compare HMAC hash
        alt wrong OTP and attempts remain
            Backend->>Moka: Store incremented attempts without extending expiry
            Backend-->>AgentApp: 401 Invalid OTP
        else max attempts reached
            Backend->>Moka: Invalidate otp_store[user_id]
            Backend-->>AgentApp: 401 Max attempts reached
        else OTP valid
            Backend->>Moka: Invalidate otp_store[user_id] (single use)
        end
    end

    Backend->>PostgreSQL: Load user's organization_id
    PostgreSQL-->>Backend: organization_id
    Backend->>Moka: Get org_work_time[organization_id]
    alt cache miss
        Backend->>PostgreSQL: SELECT start_work_time, end_work_time
        PostgreSQL-->>Backend: start/end work minutes
        Backend->>Moka: Cache org_work_time[organization_id]
    end

    Backend->>Backend: Compute today's shift_start and shift_end in UTC+1
    Backend->>Backend: Issue RS256 access token (15 min, includes shift_start/shift_end/deviceId)
    Backend->>PostgreSQL: Confirm registered device exists and suspended_at IS NULL
    PostgreSQL-->>Backend: device exists
    Backend->>PostgreSQL: Check for valid unrevoked refresh token for deviceId
    PostgreSQL-->>Backend: exists or not

    alt no valid refresh token
        Backend->>Backend: Generate raw refresh token and SHA-256 hash
        Backend->>PostgreSQL: Insert refresh token hash, set device ACTIVE, store shift metadata
        PostgreSQL-->>Backend: saved
        Backend-->>AgentApp: accessToken, refreshToken, shiftEnd
    else valid refresh token already exists
        Backend->>PostgreSQL: Mark device ACTIVE and store shift metadata
        PostgreSQL-->>Backend: updated
        Backend-->>AgentApp: accessToken, refreshToken=null, shiftEnd
    end
```

---

## Flow 3 - During the Shift: Protected Requests

Every protected mobile request validates the JWT and then checks PostgreSQL for
the user's current status, device binding/status, and token blacklist state.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant PostgreSQL

    AgentApp->>Backend: Protected request with Bearer access token
    Backend->>Backend: Verify RS256 JWT signature and exp
    Backend->>Backend: Reject if now > token.shift_end
    Backend->>PostgreSQL: Load auth validation context (blacklist, user status, active device)
    PostgreSQL-->>Backend: is_blacklisted, user_status, device_is_active
    Backend->>Backend: Require user ACTIVE and device ACTIVE/bound
    Backend-->>AgentApp: Protected resource response
```

---

## Flow 4 - Access Token Renewal

Agent refresh is a two-step challenge-response flow. The nonce is stored in Moka
for 60 seconds and consumed when verified, so replaying the same signed nonce
does not work.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant Moka as Moka AppCache
    participant PostgreSQL

    note over AgentApp,PostgreSQL: Step 1 - request refresh challenge

    AgentApp->>Backend: POST /auth/refresh (refreshToken, deviceId)
    Backend->>Backend: SHA-256 hash refreshToken
    Backend->>PostgreSQL: Find unrevoked, unexpired token for deviceId
    PostgreSQL-->>Backend: token valid
    Backend->>Backend: Generate random nonce
    Backend->>Moka: Store refresh_nonce[deviceId] = nonce (TTL 60s)
    Backend-->>AgentApp: nonce

    note over AgentApp,PostgreSQL: Step 2 - verify signed challenge

    AgentApp->>AgentApp: Sign nonce with device private key
    AgentApp->>Backend: POST /auth/refresh/verify (refreshToken, deviceId, signedNonce)
    Backend->>Moka: Read refresh_nonce[deviceId]
    Moka-->>Backend: nonce
    Backend->>Moka: Invalidate refresh_nonce[deviceId]
    Backend->>PostgreSQL: Validate refresh token again
    PostgreSQL-->>Backend: user_id
    Backend->>PostgreSQL: Load ACTIVE device public_key and shift metadata
    PostgreSQL-->>Backend: public_key, shift_start, shift_end
    Backend->>Backend: Reject if now > shift_end and mark device INACTIVE
    Backend->>Backend: Verify ES256 signed nonce with device public key
    Backend->>PostgreSQL: Load user role
    PostgreSQL-->>Backend: user
    Backend->>Backend: Issue new 15-minute access token with same shift bounds
    Backend-->>AgentApp: new accessToken
```

---

## Flow 5 - End of Shift

When the JWT `shift_end` is in the past, the backend marks the device as
`INACTIVE` and sets `revoked_at = NOW()`. Subsequent requests and refreshes fail
until the agent completes the daily login flow again.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant PostgreSQL

    AgentApp->>Backend: Protected request or /auth/refresh/verify
    Backend->>Backend: Detect now > shift_end
    Backend->>PostgreSQL: UPDATE devices SET status='INACTIVE', revoked_at=NOW()
    PostgreSQL-->>Backend: updated
    Backend-->>AgentApp: 401 Shift has ended
```

---

## Flow 6 - Admin Terminates an Agent Session

The current admin endpoint is `/api/v1/admin/terminate-session`. It revokes all
active refresh tokens for the agent and marks active devices as `INACTIVE`.
Because protected mobile requests require the device to be `ACTIVE`, the next
request from the agent is rejected.

```mermaid
sequenceDiagram
    autonumber
    participant Admin
    participant Backend
    participant PostgreSQL
    participant AgentApp

    Admin->>Backend: POST /admin/terminate-session (userId)
    Backend->>PostgreSQL: Verify target user exists and is an agent
    PostgreSQL-->>Backend: agent
    Backend->>PostgreSQL: Revoke all active refresh tokens for userId
    Backend->>PostgreSQL: Set ACTIVE devices to INACTIVE and revoked_at=NOW()
    PostgreSQL-->>Backend: transaction committed
    Backend-->>Admin: Session terminated

    AgentApp->>Backend: Next protected request with old access token
    Backend->>PostgreSQL: Validate user/device context
    PostgreSQL-->>Backend: device_is_active=false
    Backend-->>AgentApp: 401 Device is not active or not bound to user

    AgentApp->>Backend: POST /auth/request-daily-login same day
    Backend->>PostgreSQL: Load device(status, revoked_at)
    PostgreSQL-->>Backend: revoked_at is today
    Backend-->>AgentApp: 403 Wait until next shift to request a new code
```

---

## Session Rules

| Rule | Current behavior |
| ---- | ---------------- |
| Daily login request payload | `badgeId` and `deviceId` |
| OTP cache | Moka `otp_store`, keyed by `user_id` |
| OTP validity window | 5 minutes, absolute TTL |
| OTP attempts | 5 attempts max; entry is invalidated after max attempts or success |
| OTP request rate limit | 3 requests per phone number per 10 minutes in Moka `rate_limit` |
| Work window | Organization `start_work_time` and `end_work_time`, minutes since midnight, checked in UTC+1 local time |
| Work time cache | Moka `org_work_time`, loaded at startup and on cache miss |
| Access token duration | 15 minutes |
| Shift bounds | Stored in the access token and in `devices.metadata` |
| Refresh token duration | 30 days |
| Refresh flow | Challenge-response: nonce in Moka, device signs nonce, backend verifies with stored public key |
| Device activation | Successful OTP verification sets device to `ACTIVE` |
| End of shift | Backend marks device `INACTIVE` and sets `revoked_at` |
| Admin session termination | Revokes refresh tokens and marks active devices `INACTIVE`; same-day OTP request is blocked by `revoked_at` |

---

## Implementation References

| Area | File |
| ---- | ---- |
| App cache definitions | `iviss-backend/src/app_cache.rs` |
| OTP generation, hashing, rate limit, validation | `iviss-backend/src/services/otp_service.rs` |
| Daily login handlers | `iviss-backend/src/handlers/auth.rs` |
| Refresh challenge-response handlers | `iviss-backend/src/handlers/auth.rs` |
| Mobile auth middleware | `iviss-backend/src/middleware/auth.rs` |
| Auth/session queries | `iviss-backend/src/queries/auth_queries.rs`, `iviss-backend/src/queries/session_queries.rs` |
