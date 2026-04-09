-- Fix: Convert TIMESTAMP to TIMESTAMPTZ to match Rust's OffsetDateTime mapping in sqlx
-- This resolves the "mismatched types" error when loading data into OffsetDateTime structs.

-- 1. Pending Submissions
ALTER TABLE pending_submissions 
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN reviewed_at TYPE TIMESTAMPTZ;

-- 2. Organizations
ALTER TABLE organizations 
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;

-- 3. Users
ALTER TABLE users 
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;

-- 4. Vehicles
ALTER TABLE vehicles
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;

-- 5. Vehicle Owners
ALTER TABLE vehicle_owners
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;

-- 6. Vehicle Statuses
ALTER TABLE vehicle_statuses
  ALTER COLUMN last_updated TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- 7. Control Records
ALTER TABLE control_records 
  ALTER COLUMN timestamp TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;

-- 8. Control Actions
ALTER TABLE control_actions 
  ALTER COLUMN timestamp TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- 9. Devices
ALTER TABLE devices 
  ALTER COLUMN last_seen_at TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN revoked_at TYPE TIMESTAMPTZ;

-- 10. Refresh Tokens
ALTER TABLE refresh_tokens 
  ALTER COLUMN expires_at TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN revoked_at TYPE TIMESTAMPTZ;

-- 11. Audit Logs
ALTER TABLE audit_logs
  ALTER COLUMN created_at TYPE TIMESTAMPTZ;

-- 12. Access Token Blacklist
ALTER TABLE access_token_blacklist
  ALTER COLUMN expires_at TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ;
