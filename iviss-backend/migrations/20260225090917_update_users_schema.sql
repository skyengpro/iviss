-- =============================================================
-- Migration: 20260225090917_update_users_schema.sql
-- Description: modified users table
-- =============================================================
-- ─────────────────────────────────────────
-- ENUMs
-- ─────────────────────────────────────────
CREATE TYPE user_role AS ENUM (
    'admin',
    'manager',
    'agent'
);

CREATE TYPE user_status AS ENUM (
    'PENDING_ACTIVATION',
    'ACTIVE',
    'SUSPENDED'
);

-- ─────────────────────────────────────────
-- Modify the users table
-- ─────────────────────────────────────────

-- Drop the old role constraint (CHECK on VARCHAR)
ALTER TABLE users
    DROP CONSTRAINT IF EXISTS chk_user_role;

-- Migrate the role column to the new ENUM
ALTER TABLE users
    ALTER COLUMN role TYPE user_role
    USING role::user_role;

-- Add status (replaces is_active)
ALTER TABLE users
    ADD COLUMN status user_status NOT NULL DEFAULT 'PENDING_ACTIVATION';

-- Drop is_active (replaced by status)
ALTER TABLE users
    DROP COLUMN is_active;

-- Add phone_number
ALTER TABLE users
    ADD COLUMN phone_number VARCHAR(20) UNIQUE NOT NULL;

-- Make email nullable
ALTER TABLE users
    ALTER COLUMN email DROP NOT NULL;

-- Make password_hash nullable
ALTER TABLE users
    ALTER COLUMN password_hash DROP NOT NULL;

-- ─────────────────────────────────────────
-- New business constraints
-- ─────────────────────────────────────────

ALTER TABLE users
    ADD CONSTRAINT chk_users_email_required
        CHECK (
            role = 'agent'
            OR (role IN ('admin', 'manager') AND email IS NOT NULL)
        );

ALTER TABLE users
    ADD CONSTRAINT chk_users_agent_no_password
        CHECK (
            role != 'agent'
            OR (role = 'agent' AND password_hash IS NULL)
        );

-- ─────────────────────────────────────────
-- Update indexes
-- ─────────────────────────────────────────

-- Drop old indexes
DROP INDEX IF EXISTS idx_users_organization_role;
DROP INDEX IF EXISTS idx_users_email;

-- Recreate with new conditions
CREATE INDEX idx_users_phone
    ON users(phone_number)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_email
    ON users(email)
    WHERE deleted_at IS NULL AND email IS NOT NULL;

CREATE INDEX idx_users_org_role
    ON users(organization_id, role)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_status
    ON users(status)
    WHERE deleted_at IS NULL;
