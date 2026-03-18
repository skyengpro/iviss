-- Add migration script here
-- =============================================================
-- Migration: 20260317000001_admin_no_org.sql
-- Description: Makes organization_id nullable.
--              Admins are global — they don't belong to any org.
--              Agents and managers must still have an organization.
-- =============================================================

-- Allow NULL organization_id
ALTER TABLE users
    ALTER COLUMN organization_id DROP NOT NULL;

-- Enforce: only admins can have no organization
ALTER TABLE users
    ADD CONSTRAINT chk_users_org_required
        CHECK (
            role = 'admin'
            OR (role IN ('agent', 'manager') AND organization_id IS NOT NULL)
        );