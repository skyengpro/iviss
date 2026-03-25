# Manual Testing Guide: Admin-Only RBAC

This guide explains how to manually verify the Admin-Only Role-Based Access Control (RBAC) system in the IVISS project.

## 1. Bootstrap Admin Creation

The system automatically seeds an initial admin user on startup if the database is empty or no admin exists.

### Admin Bootstrap Verification

1. Ensure the following environment variables are set in `iviss-backend/.env`:
   - `ADMIN_BOOTSTRAP_EMAIL`
   - `ADMIN_BOOTSTRAP_PASSWORD`
   - `ADMIN_BOOTSTRAP_USERNAME`

2. Start the backend: `docker-compose up iviss-backend`.

3. Check the logs for: `Running admin bootstrap seed...`.

4. Verify the user exists in the database:

   ```bash
   docker exec iviss-db psql -U iviss_user -d iviss_db -c "SELECT username, role FROM users WHERE role = 'admin';"
   ```

5. Restart the backend and verify no duplicate users are created (idempotency check).

## 2. Pre-seeded Users

The system includes pre-seeded manager and agent accounts for immediate testing.

### Seeding the Database

If you need to reset or fully populate the database with these users, run:

```bash
docker exec -i iviss-db psql -U iviss_user -d iviss_db < iviss-backend/seeds/seed_data.sql
```

### Pre-configured Credentials

- **Admin**: `admin@iviss.gov` / `admin123`
- **Manager 1**: `manager@iviss.gov` / `admin123`
- **Manager 2**: `manager2@iviss.gov` / `admin123`

## 3. Admin Authentication

Verify that administrative users can log in and receive a token with the correct role claim.

### Admin Authentication Verification

1. Perform a login request via `curl`:

   ```bash
   curl -X POST http://localhost:3000/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "admin@iviss.gov", "password": "admin123"}'
   ```

2. Confirm the response contains:
   - `accessToken` (JWT)
   - `user` object with `"role": "admin"`

3. (Optional) Paste the `accessToken` into [jwt.io](https://jwt.io) and verify the `role` claim in the payload is `"admin"`.

## 4. Creating a Manager User for Testing

To test the 403 Forbidden scenarios, you need a user with the `manager` role.

### Step 1: Provision the Manager via Admin API

Use your Admin token to create the manager:

```bash
curl -X POST http://localhost:3000/admin/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <ADMIN_TOKEN>" \
  -d '{
    "username": "test_manager",
    "phoneNumber": "+237600000002",
    "fullName": "Test Manager",
    "role": "manager",
    "organizationId": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "email": "manager@test.com",
    "badgeId": "MGR-999"
  }'
```

*Note: Ensure the `organizationId` exists in your database.*

### Step 2: Activate and Set Password (Manual)

Since the provision API creates users in `PENDING_ACTIVATION` status without a password, use `psql` to set them up for testing:

```bash
# Set password to 'admin123' and status to 'ACTIVE'
docker exec iviss-db psql -U iviss_user -d iviss_db -c "UPDATE users SET status = 'ACTIVE', password_hash = '\$argon2id\$v=19\$m=19456,t=2,p=1\$owHhGNruIX1moa5B1514cA\$7Cz1awNhXhvwzYsaGOiooLs+zWNUqk9NXL8SxuvvLpQ' WHERE email = 'manager@test.com';"
```

### Step 3: Login as Manager

Navigate to `http://localhost:8080/admin-login` and use the manager credentials.

*Note: The "Admin login" button on the Daily Login page correctly redirects you to this screen.*

Confirm the response (visible in network logs or after redirect) shows the user has the `"role": "manager"`.

## 5. Backend Enforcement (RBAC)

Verify that the backend correctly protects admin-only endpoints.

### Setup

- **Admin Token**: Obtain via login as above.
- **Manager Token**: Create a manager user via the admin API and log in as them.
- **No Token**: Test without the Authorization header.

### Test Cases

| Scenario | Endpoint | Expected Result |
| :--- | :--- | :--- |
| Unauthenticated | `GET /admin/users` | `401 Unauthorized` |
| Non-Admin User (Manager) | `GET /admin/users` | `403 Forbidden` |
| Admin User | `GET /admin/users` | `200 OK` |

**Example Curl (Unauthorized)**

```bash
curl -X GET http://localhost:3000/admin/users -H "Authorization: Bearer <MANAGER_TOKEN>"
```

## 6. Frontend Gating

Verify that the UI correctly restricts access based on the user's role.

### Sidebar Visibility

1. Log in to the Back-Office as a **Manager**.
2. Verify that the **Administration** section (User Management, Audit Logs, Organizations) is **HIDDEN** in the sidebar.
3. Log in as an **Admin**.
4. Verify that the **Administration** section is **VISIBLE**.

### Route Protection

1. While logged in as a **Manager**, try to navigate directly to:
   - `http://localhost:8080/backoffice/users`

2. Verify that the application redirects you back to the main dashboard or mobile page instead of showing the user management screen.

## 7. End-to-End User Management (Admin Only)

As an Admin, perform a full user lifecycle task:

1. Navigate to **User Management**.
2. Click **Add User**.
3. Create a new **Agent** user.
4. Verify the user appears in the list.
5. (Optional) Try to perform the same action as a Manager (it should be blocked by the UI and the API).
