# Testing Guide: Admin Session Management & Authentication

## Overview
This feature introduces remote session management, allowing administrators to terminate or restart an agent's session. It also handles the tricky difference between registering a device (Activation) and logging into a registered device (Daily Login).

---

## 💡 Enlightenment: Activation vs. Daily Login
You mentioned you keep getting **"Device is not registered"** when using `activation-code.sh`. Here is the enlightenment:

- **Activation (`/auth/activate`)**: This is when an Agent connects a device for the **very first time**. The backend saves the device's unique ID into the database. (In production, this requires an SMS sent by an Admin).
- **Daily Login (`/auth/request-daily-login`)**: This is when an Agent comes to work the next day. The backend strictly checks if the **exact same Device ID** is already in the database.

**Why `activation-code.sh` fails**: Your `activation-code.sh` script generates a *random newly created UUID* every time it runs, and sends it to the **Daily Login** endpoint. Because that random ID is not in the database, the backend correctly rejects it with a **404 Not Found (Device is not registered)**. 

To fix this, you should just use the actual Web Browser! The browser automatically handles saving and sending the correct Device ID. If you truly want to use a script, you must use the Admin Activation endpoint to register the device first, and then save the generated UUID to use for future Daily Logins.

---

## 1. How the Agent Login Flow Works (In the Browser)

### Step 1: Device Activation (The first time ever)
1. Open up the browser and go to `http://localhost:8080`.
2. As an Agent, entering a badge number and requesting an OTP here will tell the Admin you need access.
3. The Admin logs in to the backoffice and clicks "Resend activation code" for your user. (In our dev environment, look at the terminal logs: `docker compose logs backend --since=2m | grep "MOCK SMS"` to find the 6-digit code).
4. Enter the code in the Agent browser to `Activate`. Your browser's unique Device ID is now officially stored in the database.

### Step 2: Daily Login (Every everyday shift)
1. The next day, the Agent goes to `http://localhost:8080/daily-login`.
2. They enter their badge number. The browser secretly sends the exact same Device ID it saved during activation.
3. Since the Device ID matches the database, the backend sends a new SMS OTP.
4. The Agent enters the OTP and gets access. 

---

## 2. Testing Admin Session Termination & Restart

For the best experience, use **two browser windows**: one normal window for the Admin, and one "Incognito/Private" window for the Agent (so their cookies/storage don't mix).

### 2.1 Test: Session Termination
1. **Agent Window**: Ensure the Agent is logged in and on their Dashboard.
2. **Admin Window**: Go to `Backoffice -> User Management`. 
3. You will see the Agent's Session status as **ACTIVE** (Green). 
4. Click the `...` Actions menu and choose **Terminate Session**. 
5. The Session status will immediately turn **INACTIVE** (Red). 
6. **Agent Window**: Refresh the page, or try to click a link. The Agent will instantly see a red warning: **"Your session has been terminated by an administrator"** and will be kicked out to the Daily Login screen.

### 2.2 Test: Stopping Re-entry (Shift Block)
If the terminated Agent tries to log back in immediately from the Daily Login screen, they will get an error: **"Outside shift hours"** or **"You cannot log in again because your session was terminated."** The system blocks them from returning the same day.

### 2.3 Test: Session Restart
If the Admin realizes it was a mistake and wants to let the Agent back in:
1. **Admin Window**: Go to User Management. Find the Agent (whose session is INACTIVE).
2. Click the `...` Actions menu and select **Restart Session**.
3. The Admin dashboard will show the Agent is **ACTIVE** again.
4. **Agent Window**: The Agent goes to the Daily Login page. Because the Admin restarted their session, they are now allowed to request a daily OTP and log back in successfully without needing a brand new device activation.

---

## 3. Fixing your `activation-code.sh` script

If you absolutely must use your `activation-code.sh` script to test the backend, you CANNOT generate a random (`uuid.uuid4()`) device ID. The backend will reject it as "Device is not registered".

**How to fix it:**
1. Open your browser and go to `http://localhost:8080/mobile`.
2. Open Developer Tools (F12) -> Application Tab -> Local Storage.
3. Find the key named `iviss_device_id` and copy the value (e.g., `550e8400-e29b-41d4-a716-446655440000`).
4. Modify your script to use that exact UUID instead of Python's random generator:

```bash
#!/bin/bash
# Put YOUR browser's real device ID here:
UUID="550e8400-e29b-41d4-a716-446655440000"

curl -i -X POST http://localhost:3000/auth/request-daily-login \
  -H "Content-Type: application/json" \
  -d "{\"badgeId\":\"AGT-102\",\"deviceId\":\"$UUID\"}"
```

This will correctly ask the backend for an OTP for your actual registered browser session!
