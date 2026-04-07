-- no-transaction
-- Add org_admin role to user_role enum (if not exists)
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'org_admin';
