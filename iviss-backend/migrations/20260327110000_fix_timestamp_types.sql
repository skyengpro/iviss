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
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- 3. Users
ALTER TABLE users 
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN last_revoked_at TYPE TIMESTAMPTZ;

-- 4. Control Records
ALTER TABLE control_records 
  ALTER COLUMN timestamp TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ,
  ALTER COLUMN deleted_at TYPE TIMESTAMPTZ;

-- 5. Control Actions
ALTER TABLE control_actions 
  ALTER COLUMN timestamp TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- 6. Devices
ALTER TABLE devices 
  ALTER COLUMN last_seen_at TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ;

-- 7. Refresh Tokens
ALTER TABLE refresh_tokens 
  ALTER COLUMN expires_at TYPE TIMESTAMPTZ,
  ALTER COLUMN created_at TYPE TIMESTAMPTZ,
  ALTER COLUMN updated_at TYPE TIMESTAMPTZ;
