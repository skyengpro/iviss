-- =============================================================
-- Migration: 20260311084804_update_user_table.sql
-- Description: Remove UNIQUE constraint from username; add UNIQUE constraint to badge_id
-- =============================================================

-- Drop the UNIQUE constraint on username
-- (created as inline UNIQUE in the initial schema, so its system-generated name must be used)
ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_username_key;

-- Make badge_id NOT NULL and UNIQUE
ALTER TABLE users
    ALTER COLUMN badge_id SET NOT NULL;

ALTER TABLE users
    ADD CONSTRAINT users_badge_id_key UNIQUE (badge_id);
