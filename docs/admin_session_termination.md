# Testing Guide: Admin Session Termination & Auth Fixes

## Overview

This feature introduces remote session termination allowing administrators to forcefully log out agents from their active devices. It also fixes critical authentication issues, including JWT RSA key validation and agent re-activation logic.

## 1. Environment Setup

### 1.1 Generating RSA Keys

You must have a matching 2048-bit RSA key pair for the backend to sign and verify JWTs correctly.

Run the following commands in your terminal to generate them:

```bash
# Generate private key
openssl genrsa -out private.pem 2048

# Extract public key
openssl rsa -in private.pem -pubout -out public.pem
```

### 1.2 Updating `.env` Files

You need to copy the contents of the generated keys into **two** `.env` files:

1. `iviss/.env`
2. `iviss/iviss-backend/.env`

Open the `.env` files and paste the raw content of the keys, wrapping them in quotes and replacing newlines with `\n`, or use standard multiline format if your docker compose setup supports it.

Example format:

```env
JWT_PRIVATE_KEY_PEM="-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----"
JWT_PUBLIC_KEY_PEM="-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"
```

_(Make sure there are absolutely NO typos in the keys compared to what `openssl` outputted.)_

**Important:** Rebuild your backend container after updating the `.env` files:

```bash
docker compose up -d --build backend
```

## 2. Testing Steps

### Step 1: Login as Administrator

1. Open a normal browser window and navigate to the IVISS frontend (e.g., `http://localhost:8080`).
2. On the main activation screen, click the **"Admin login"** button at the bottom.
3. Log in using the admin credentials:
   - **Email/Username**: `admin01`
   - **Password**: `admin123`
4. Submit the form. You will be taken directly to the **Admin Dashboard** (there is no OTP or code required for this admin dev flow).
5. Keep this window open.

### Step 2: Trigger Agent Activation Code via cURL

Because the activation endpoint requires an Admin JWT, it's easiest to trigger it via terminal.

1. Retrieve your Admin token from the browser (Check `localStorage` for `iviss_session`, or run the command below which does the login for you).
2. Run this command in your terminal to trigger the OTP for Agent `Michael Johnson` (Badge: `AGT-104`, User ID: `f690f1ee-6c54-4b01-90e6-d701748f0855`):

```bash
# 1. Login to get the Admin Token
TOKEN=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin01","password":"admin123"}' | grep -o '"token":"[^"]*' | cut -d'"' -f4)

# 2. Trigger the Activation SMS/OTP
curl -X POST http://localhost:3000/auth/send-activation \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"user_id": "f690f1ee-6c54-4b01-90e6-d701748f0855"}'
```

_(Note: If the response is `{"message":"Activation code sent successfully"}`, proceed to the next step.)_

### Step 3: Retrieve the OTP

In local development, the SMS provider is mocked. You must fetch the OTP from the backend logs:

```bash
docker compose logs backend --since=2m | grep "MOCK SMS"
```

You will see a log line containing the 6-digit code.

### Step 4: Login as the Agent (Incognito)

1. Open an **Incognito / Private Browsing window** (this prevents mixing sessions with your Admin window).
2. Navigate to the login page (`http://localhost:8080`).
3. Enter the agent details:
   - **Badge ID**: `AGT-104`
   - **OTP**: _(The 6-digit code you retrieved from the logs)_
4. Click Activate. You should now be logged in as the Agent.

### Step 5: Test Admin Session Termination

1. Go back to your **Admin window** (the normal browser window).
2. Navigate to **User Management**.
3. Locate `Michael Johnson` in the user list.
4. Click the Actions menu (three dots) on their row and select **Terminate Session**.
5. Confirm the action in the popup.

### Step 6: Verify Forced Logout

1. Switch back to your **Agent window** (Incognito window).
2. Observe the screen. Within a moment (or upon clicking any link/action), the application will:
   - Display a "Session Terminated" notification.
   - Automatically redirect you to the login screen.
3. Open Developer Tools (F12) -> Application Tab -> **IndexedDB**.
4. Verify that the `EventKeyStorage` database is completely empty (both `keys` and `metadata` tables are cleared).

### Step 7: Verify Re-activation (Regression Check)

1. In your terminal, run the exact same `send-activation` cURL command from **Step 2**.
2. It should succeed again. _(This proves that agents are not permanently blocked from logging in after termination)._
