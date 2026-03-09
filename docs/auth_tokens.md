# Access & Refresh Tokens (Backend)

This document explains how IVISS generates and verifies authentication tokens in the backend.

## Overview

IVISS uses two token types:

- **Access token**: a short-lived **JWT** signed with **RS256**.
- **Refresh token**: a long-lived **opaque random string** stored server-side as a **SHA-256 hash**.

The current production-ready flow is implemented in the **activation** endpoint:

- `POST /auth/activate` issues both an access token and a refresh token.

## Configuration / Secrets

Tokens depend on the following environment variables (see `docker-compose.yml`):

- `JWT_PRIVATE_KEY_PEM`
  - RSA private key used to **sign** access tokens.
- `JWT_PUBLIC_KEY_PEM`
  - RSA public key used to **verify** access tokens.

These values are loaded into `AppState`:

- `iviss-backend/src/app_state.rs`
  - `jwt_private_key_pem: String`
  - `jwt_public_key_pem: String`

## Access token (JWT)

### Where it is generated

- `iviss-backend/src/handlers/auth.rs`
  - In `activate(...)`, after activation and device registration succeed.

Relevant code path:

1. Create `JwtService` from the RSA private key PEM:
   - `JwtService::new(&state.jwt_private_key_pem)`
2. Issue the access token:
   - `jwt_svc.issue_access_token(user_id, payload.device_id, user.role)`

### How it is generated

- `iviss-backend/src/services/jwt_service.rs`

The service defines:

- `ACCESS_TOKEN_TTL`: `15 minutes`
- `AccessTokenClaims`:
  - `sub`: user id (`Uuid`)
  - `device_id`: device id (`Uuid`)
  - `role`: user role string
  - `exp`: expiration timestamp (seconds since epoch)
  - `jti`: random token id (`Uuid::new_v4()`)

Token signing:

- Algorithm: **RS256**
- Library: `jsonwebtoken`
- Signing key: `EncodingKey::from_rsa_pem(jwt_private_key_pem.as_bytes())`
- Header type is set to `JWT`.

### How it is stored

The access token is **not stored** in the database. It is verified on each request.

This makes the access token **stateless**.

## Refresh token (opaque)

### Where it is generated

- `iviss-backend/src/handlers/auth.rs`
  - In `activate(...)`, before returning the response.

### How it is generated

The refresh token is generated as:

- 32 cryptographically-random bytes
- URL-safe Base64 encoding without padding

Code:

- `rand::thread_rng()`
- `rng.fill_bytes(&mut raw)`
- `base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)`

### How it is stored

The backend stores a **SHA-256 hash** of the refresh token in the DB:

- Hashing:
  - `sha2::Sha256::digest(refresh_token.as_bytes())`
  - Stored as lowercase hex string via `format!("{:x}", digest)`

- Storage table:
  - `refresh_tokens`

- Insert statement:
  - `INSERT INTO refresh_tokens (token_hash, user_id, device_id, expires_at) VALUES ($1, $2, $3, $4)`

- Expiration:
  - `expires_at` is set to `now + 30 days`

Important security property:

- The **plaintext refresh token is never stored** server-side.
- Only the **hash** is stored.

### What is returned to the frontend

The refresh token plaintext is returned once in the activation response:

- Response DTO: `ActivateResponse`
  - `access_token`
  - `refresh_token`
  - `user`

After that, the client must keep it. If lost, it cannot be recovered from the server.

## Token verification on API requests

### How access tokens are verified

For endpoints that require authentication, the backend extracts and verifies the access token from:

- `Authorization: Bearer <access_token>`

Implementation:

- `iviss-backend/src/middleware/auth.rs`
  - `AuthUser` extractor implements `FromRequestParts<Arc<AppState>>`

Steps:

1. Read the `Authorization` header.
2. Enforce the `Bearer ` scheme.
3. Parse the RSA public key:
   - `DecodingKey::from_rsa_pem(state.jwt_public_key_pem.as_bytes())`
4. Verify and decode the JWT:
   - `decode::<AccessTokenClaims>(token, &decoding_key, &Validation::new(Algorithm::RS256))`
5. Validate expiration (`exp`).
6. Populate `AuthUser`:
   - `user_id` from `claims.sub`
   - `device_id` from `claims.device_id`
   - `role` from `claims.role`

If any step fails, the request is rejected with:

- `401 Unauthorized`

### Example: `GET /users/me`

- `iviss-backend/src/handlers/users.rs`

The handler signature includes `auth: AuthUser`.

It uses:

- `auth.user_id`

to load and return the authenticated user profile.

## Refresh token verification / rotation

A refresh endpoint (typically `POST /auth/refresh`) would verify refresh tokens by:

1. Hashing the presented refresh token with SHA-256
2. Looking up `refresh_tokens.token_hash` where:
   - `user_id` matches
   - `device_id` matches
   - `expires_at` is in the future
   - `revoked = false` (if present in schema)
3. If valid:
   - Issue a new access token (JWT)
   - Optionally rotate refresh token (generate a new one and revoke the old)

Note:

- In the current codebase snapshot, activation stores refresh tokens, but there is not yet a dedicated refresh endpoint implemented.

## Testing reference

There is an integration test validating that refresh tokens are stored hashed:

- `iviss-backend/src/tests/activation_endpoint_tests.rs`

It checks:

- returned `refreshToken` is non-empty
- stored `token_hash` equals `sha256(refreshToken)`
- stored `token_hash` is not equal to the plaintext token
