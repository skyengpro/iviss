# Daily Login OTP Flow

## Overview

The Daily Login Flow allows agents to authenticate at the start of each shift using a one-time password (OTP) sent via SMS. Upon successful verification, a shift-scoped access token (8 hours) is issued. When the shift token expires, the session is automatically cleared.

This flow is described in `User_registration.md` Section 4 and is separate from the initial device activation/registration flow.

---

## Architecture

```
┌───────────────┐       POST /auth/request-daily-login       ┌─────────────────┐
│               │  ────────────────────────────────────────▶ │                 │
│   Frontend    │       { phone_number, device_id }          │    Backend      │
│  DailyLogin   │                                            │  daily_login.rs │
│    Page       │  ◀──────────────────────────────────────── │                 │
│               │       { message, expires_in: 300s }        │  ┌────────────┐ │
│               │                                            │  │ DailyOtp   │ │
│               │       POST /auth/verify-daily-login        │  │ Service    │ │
│               │  ────────────────────────────────────────▶ │  │            │ │
│               │       { phone_number, otp, device_id }     │  │ Redis+HMAC │ │
│               │                                            │  └────────────┘ │
│               │  ◀──────────────────────────────────────── │                 │
│               │       { access_token, expires_in: 28800s } │  ┌────────────┐ │
│               │                                            │  │ SMS        │ │
└───────────────┘                                            │  │ Provider   │ │
                                                             │  └────────────┘ │
                                                             └─────────────────┘
```

---

## User Flow

1. Agent logs in with credentials (existing login flow → `/login`)
2. Agent navigates to `/daily-login`
3. **Phase 1 — Request OTP**: Agent clicks "Request OTP"
   - Frontend sends `POST /auth/request-daily-login` with the agent's phone number and device ID
   - Backend verifies user is `ACTIVE`, generates a 6-digit OTP, stores its HMAC-SHA256 hash in Redis (5-min TTL), and sends the code via SMS
   - Frontend starts a 5-minute countdown timer
4. **Phase 2 — Verify OTP**: Agent enters the 6-digit code
   - Frontend sends `POST /auth/verify-daily-login`
   - Backend validates the OTP against the Redis entry
   - On success: returns shift tokens (8-hour access token)
   - Frontend stores the shift token in `sessionStorage` and `AuthContext`
5. **Phase 3 — Success**: Agent is redirected to the mobile dashboard
6. **Shift Expiration**: After 8 hours, the `AuthContext` auto-clears the shift session

---

## Backend Components

### `DailyOtpService` — `src/services/daily_otp_service.rs`

| Method | Description |
|---|---|
| `generate_code()` | Generates a random 6-digit zero-padded numeric code |
| `hash_code(code)` | HMAC-SHA256 hash using the application pepper (`ACTIVATION_CODE_PEPPER`) |
| `generate_and_store(user_id)` | Generates OTP, stores hash in Redis with `daily_otp:{user_id}` key, 300s TTL |
| `validate(user_id, otp)` | Validates submitted OTP, tracks attempts (max 5), deletes key on success or max attempts |
| `generate_and_send(user_id, phone)` | Orchestrates generate + store + SMS send |

**Redis key format**: `daily_otp:{user_id}`  
**TTL**: 300 seconds (5 minutes)  
**Max attempts**: 5

### Handler — `src/handlers/daily_login.rs`

| Endpoint | Method | Description |
|---|---|---|
| `/auth/request-daily-login` | POST | Validates user is ACTIVE, generates and sends OTP |
| `/auth/verify-daily-login` | POST | Validates OTP, returns shift access token |

**Request/Response DTOs**: `RequestDailyLoginRequest`, `RequestDailyLoginResponse`, `VerifyDailyLoginRequest`, `VerifyDailyLoginResponse`

---

## Frontend Components

### `DailyLogin.tsx` — `src/pages/auth/DailyLogin.tsx`

Three-phase page component:
- **Phase 1 (request)**: "Request OTP" button with smartphone icon
- **Phase 2 (verify)**: 6-digit OTP input with individual digit boxes, countdown timer with color transitions (green → amber → red), "Verify & Start Shift" button
- **Phase 3 (success)**: Animated checkmark, auto-redirect to `/mobile` after 1.5s

**i18n**: Fully internationalized (English + French) via `react-i18next` under the `dailyLogin` namespace.

### `AuthContext.tsx` — Shift Session Management

| Property/Method | Description |
|---|---|
| `shiftToken` | Current shift access token (or null) |
| `shiftExpiresAt` | Expiration timestamp (or null) |
| `isShiftActive` | Computed boolean — true if token exists and hasn't expired |
| `setShiftSession(token, expiresIn)` | Stores token in state + `sessionStorage`, schedules auto-expiration |
| `clearShiftSession()` | Clears token from state + `sessionStorage`, cancels timer |

**Persistence**: `sessionStorage` (survives page refreshes within the same tab, cleared on tab close).  
**Auto-expiration**: A `setTimeout` is scheduled for the exact duration. On expiry, `clearShiftSession()` is called automatically.

---

## API Reference

### POST `/auth/request-daily-login`

**Request:**
```json
{
  "phone_number": "+254700123457",
  "device_id": "1ed24df4-6add-418a-ae09-7a595f7b6a52"
}
```

**Response (201):**
```json
{
  "message": "Daily OTP sent successfully",
  "expires_in": 300
}
```

**Errors:**
- `404` — User not found
- `400` — User not active

### POST `/auth/verify-daily-login`

**Request:**
```json
{
  "phone_number": "+254700123457",
  "otp": "103712",
  "device_id": "1ed24df4-6add-418a-ae09-7a595f7b6a52"
}
```

**Response (200):**
```json
{
  "access_token": "shift-jwt-xxxx-xxxx",
  "refresh_token": "shift-refresh-xxxx-xxxx",
  "expires_in": 28800,
  "token_type": "Bearer"
}
```

**Errors:**
- `401` — Invalid or expired OTP / Max attempts reached
- `404` — User not found

---

## Testing

### Prerequisites

1. Backend and all services running: `docker compose up -d`
2. Seed data loaded: `docker exec -i iviss-db psql -U iviss_user -d iviss_db < ./iviss-backend/seeds/seed_data.sql`
3. Test user: `agent1` with phone `+254700123457` (status: ACTIVE)

### Manual Testing

1. **Log in** as `agent01` / `agent123` at `http://localhost:8080/login`
2. Navigate to `http://localhost:8080/daily-login`
3. Click **"Request OTP"** — verify the countdown timer starts (5:00)
4. Read the OTP from backend logs: `docker logs -f iviss-backend`
   - Look for: `[MOCK SMS] — message not actually sent phone=+254700123457 ... est : XXXXXX`
5. Enter the 6-digit code in the input boxes
6. Click **"Verify & Start Shift"** — verify:
   - Success animation appears
   - Redirect to `/mobile` dashboard after 1.5s
   - `sessionStorage` contains `iviss_shift_token` and `iviss_shift_expires_at`

### Testing Error Cases

| Scenario | How to Test | Expected Result |
|---|---|---|
| Wrong OTP | Enter `000000` instead of the real code | "Invalid or expired OTP" error (translated) |
| Expired OTP | Wait for the 5-minute countdown to reach 0:00 | Timer resets to Phase 1, "OTP expired" message |
| Max attempts | Enter wrong OTP 5 times | OTP invalidated in Redis, error displayed |
| Inactive user | Change user status to `SUSPENDED` in DB | "Your account is currently inactive..." (translated) |
| Language switch | Set `localStorage.setItem('i18nextLng', 'fr')` and refresh | All UI text displays in French |

### API Testing with cURL

```bash
# Request OTP
curl -X POST http://localhost:3000/auth/request-daily-login \
  -H "Content-Type: application/json" \
  -d '{"phone_number": "+254700123457", "device_id": "test-device"}'

# Verify OTP (replace XXXXXX with code from backend logs)
curl -X POST http://localhost:3000/auth/verify-daily-login \
  -H "Content-Type: application/json" \
  -d '{"phone_number": "+254700123457", "otp": "XXXXXX", "device_id": "test-device"}'
```

### Swagger UI

Both endpoints are registered in the OpenAPI spec. Visit `http://localhost:3000/docs` to test interactively.

---

## Configuration

| Environment Variable | Description | Default |
|---|---|---|
| `ACTIVATION_CODE_PEPPER` | Secret pepper for HMAC-SHA256 OTP hashing | Required (dev default in docker-compose) |
| `REDIS_URL` | Redis connection for OTP storage | `redis://iviss-redis:6379` |
| `VITE_API_URL` | Frontend API base URL | `http://localhost:3000` |

---

## Files Changed

### Backend
- `src/services/daily_otp_service.rs` — OTP generation, hashing, Redis storage, validation
- `src/services/mod.rs` — Module registration
- `src/handlers/daily_login.rs` — Request and verify handlers with DTOs
- `src/handlers/mod.rs` — Module registration
- `src/routes.rs` — Two new POST routes
- `src/api_doc.rs` — OpenAPI registration
- `docker-compose.yml` — Added `ACTIVATION_CODE_PEPPER` env var

### Frontend
- `src/pages/auth/DailyLogin.tsx` — OTP page with 3-phase UI
- `src/contexts/AuthContext.tsx` — Shift session state management
- `src/hooks/auth/use-auth.ts` — Extended AuthContextType interface
- `src/router/routes.ts` — `/daily-login` route
- `src/services/mockAuth.ts` — Added `phoneNumber` field to mock users
- `src/i18n/locales/en.json` — English translations (29 keys)
- `src/i18n/locales/fr.json` — French translations (29 keys)
