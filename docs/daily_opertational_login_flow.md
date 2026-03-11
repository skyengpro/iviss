
# BE-08 — Daily Login Flow (Agent)

**Ticket:** BE-08
**Last updated:** 2026-03-06

---

## What is this?

Every morning, before going on patrol, an agent confirms their identity with a **6-digit code sent by SMS** combined with their  **badge ID** . This opens their shift and gives them access to the app until the shift ends.

At the end of the shift, access closes automatically and the device returns to standby — ready for the next morning.

An admin can suspend an agent's device at any time, which immediately blocks all access, even mid-session.

---

## Actors

| Actor                | Role                                                                |
| -------------------- | ------------------------------------------------------------------- |
| **AgentApp**   | The Android app on the agent's phone                                |
| **Backend**    | The IVISS server                                                    |
| **Redis**      | Temporary storage — OTP codes, rate limits, blacklisted tokens     |
| **PostgreSQL** | Main database — users, devices, tokens                             |
| **SMS**        | Gateway (Twilio) that sends the OTP to the agent                    |
| **Admin**      | Back-office user who manages agents and can suspend/restore devices |

---

## Device Status

A device has exactly three possible states:

| Status        | Meaning                                                 | Can request OTP?                          |
| ------------- | ------------------------------------------------------- | ----------------------------------------- |
| `INACTIVE`  | Normal standby — shift not started or shift just ended | ✅ Yes                                    |
| `ACTIVE`    | Shift in progress — agent is working                   | ✅ Yes (but new OTP not needed mid-shift) |
| `SUSPENDED` | Blocked by admin                                        | ❌ No                                     |

> **Only a successful OTP + badge ID verification can set a device to `ACTIVE`.**
> The device goes back to `INACTIVE` automatically when the shift ends, or when an admin lifts a suspension.

---

## Flow 1 — Morning Login (OTP)

The agent requests their daily code, then confirms it with their badge ID.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant Redis
    participant PostgreSQL
    participant SMS

    note over AgentApp,SMS: Step 1 — Agent requests their daily code

    AgentApp->>Backend: "I want to log in" (phone number + device ID)
    Backend->>PostgreSQL: Does this agent exist and is their account not suspended?
    PostgreSQL-->>Backend: Agent found, account OK
    Backend->>PostgreSQL: Is this device registered? Is its status INACTIVE or ACTIVE (not SUSPENDED)?
    PostgreSQL-->>Backend: Device found, not suspended
    Backend->>Redis: Has this phone requested too many codes recently? (max 3 per 10 min)
    Redis-->>Backend: Within limit — OK
    Backend->>Backend: Generate a 6-digit code
    Backend->>Redis: Save the code for 5 minutes (single use)
    Redis-->>Backend: Saved
    Backend->>SMS: Send code to agent's phone
    SMS-->>AgentApp: SMS received

    note over AgentApp,SMS: Step 2 — Agent confirms with code + badge ID

    AgentApp->>Backend: "Here is my code + badge ID" (otp + badgeId + device ID)
    Backend->>PostgreSQL: Confirm agent exists and badge ID matches
    PostgreSQL-->>Backend: Confirmed
    Backend->>Redis: Retrieve saved code + time remaining
    Redis-->>Backend: Code + remaining time
    Backend->>Backend: Check code matches, not expired (5 min), under 5 attempts
    Backend->>Redis: Delete the code (single use — cannot be reused)
    Redis-->>Backend: Deleted
    Backend->>Backend: Create Access Token (valid 15 min, contains shift_start and shift_end)
    Backend->>Backend: Create one Refresh Token (valid 30 days)
    Backend->>PostgreSQL: Save Refresh Token + set device status = ACTIVE
    PostgreSQL-->>Backend: Saved
    Backend-->>AgentApp: Access Token + Refresh Token + shift end time
```

---

## Flow 2 — During the Shift (Token Renewal)

The access token lasts 15 minutes. When it expires, the app silently gets a new one using the refresh token — the agent sees nothing. The **same refresh token (30 days) is reused** throughout all renewals during the agent's sessions.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant Redis
    participant PostgreSQL

    note over AgentApp,PostgreSQL: Normal use — access token valid

    AgentApp->>Backend: Search for a plate (with Access Token)
    Backend->>Redis: Has this token been revoked?
    Redis-->>Backend: No — valid
    Backend-->>AgentApp: Vehicle result

    note over AgentApp,PostgreSQL: Access token expired — silent renewal

    AgentApp->>Backend: "My token expired, renew it" (Refresh Token + device ID)
    Backend->>PostgreSQL: Find the Refresh Token — is it valid and not blacklisted?
    PostgreSQL-->>Backend: Valid, not blacklisted
    Backend->>Backend: Check shift_end not yet reached
    Backend->>Backend: Generate a new Access Token (15 min, same shift_start and shift_end)
    Backend-->>AgentApp: New Access Token (same Refresh Token continues to be used)

```

---

## Flow 3 — End of Shift (Automatic)

When `shift_end` is reached, the system automatically closes the session. The device returns to `INACTIVE` — the agent can request a new OTP the next morning.

```mermaid
sequenceDiagram
    autonumber
    participant AgentApp
    participant Backend
    participant PostgreSQL

    note over AgentApp,PostgreSQL: shift_end timestamp reached

    AgentApp->>Backend: "My token expired, renew it" (Refresh Token)
    Backend->>Backend: shift_end already passed — session closed
    Backend->>PostgreSQL: Set device status = INACTIVE
    PostgreSQL-->>Backend: Done
    Backend-->>AgentApp: "Your shift is over — log in again tomorrow"

    note over AgentApp,PostgreSQL: Next morning — agent goes through Flow 1 again
```

---

## Flow 4 — Admin Suspends a Device

An admin can immediately cut access to a device at any time — even during an active session. The refresh token (30 days) is blacklisted so no new access token can be generated.

```mermaid
sequenceDiagram
    autonumber
    participant Admin
    participant Backend
    participant Redis
    participant PostgreSQL

    note over Admin,PostgreSQL: Admin suspends a device

    Admin->>Backend: "Suspend this device" (device ID)
    Backend->>PostgreSQL: Set device status = SUSPENDED
    PostgreSQL-->>Backend: Done
    Backend->>Redis: Blacklist the current Access Token (expires with the token naturally)
    Redis-->>Backend: Done
    Backend->>PostgreSQL: Blacklist the Refresh Token (30 days) — mark as revoked
    PostgreSQL-->>Backend: Done
    Backend-->>Admin: "Device suspended"

    note over Admin,PostgreSQL: Agent tries to use the app — blocked immediately

    AgentApp->>Backend: Any request (with Access Token)
    Backend->>Redis: Is this token blacklisted?
    Redis-->>Backend: Yes — blacklisted
    Backend-->>AgentApp: "Access denied — contact your administrator"

    note over Admin,PostgreSQL: Agent tries to request a new OTP — also blocked

    AgentApp->>Backend: "I want to log in" (phone + device ID)
    Backend->>PostgreSQL: Is this device suspended?
    PostgreSQL-->>Backend: Yes — SUSPENDED
    Backend-->>AgentApp: "Access denied — contact your administrator"
```

---

## Flow 5 — Admin Lifts the Suspension

When an admin lifts the suspension, the device goes back to `INACTIVE`. The agent must go through a normal OTP login (Flow 1) to become `ACTIVE` again.

```mermaid
sequenceDiagram
    autonumber
    participant Admin
    participant Backend
    participant PostgreSQL
    participant AgentApp

    Admin->>Backend: "Lift suspension for this device" (device ID)
    Backend->>PostgreSQL: Set device status = INACTIVE
    PostgreSQL-->>Backend: Done
    Backend-->>Admin: "Device restored — agent can log in again"

    note over Admin,PostgreSQL: Agent goes through Flow 1 (OTP + badge ID) to become ACTIVE again
    note over Admin,PostgreSQL: A new 30-day Refresh Token is created at that point
```

---

## Session Rules

| Rule                         | Detail                                                              |
| ---------------------------- | ------------------------------------------------------------------- |
| Shift is valid while         | `shift_start ≤ current time < shift_end`                         |
| Access token duration        | 15 minutes — renewed automatically using the Refresh Token         |
| Refresh token duration       | 30 days — one token per activation, reused throughout all renewals |
| A device becomes ACTIVE      | Only after successful OTP + badge ID verification                   |
| A device returns to INACTIVE | Automatically at `shift_end`, or when admin lifts a suspension    |
| A device is SUSPENDED        | Admin action — blocks OTP requests, blacklists the Refresh Token   |
| Max OTP attempts             | 5 — code destroyed after the 5th wrong attempt                     |
| OTP validity window          | 5 minutes — cannot be extended                                     |
| Max OTP requests per phone   | 3 per 10 minutes                                                    |

---

## Dependencies

| Ticket | What is needed                                            |
| ------ | --------------------------------------------------------- |
| BE-02  | OTP generation and SMS sending                            |
| BE-03  | Token creation, device registration and status management |
| BE-05  | Token validation on every protected request               |
