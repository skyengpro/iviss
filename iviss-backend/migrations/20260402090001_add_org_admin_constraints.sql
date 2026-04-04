-- Migration: 20260402090001_add_org_admin_constraints.sql

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS chk_users_org_required;

ALTER TABLE users
    ADD CONSTRAINT chk_users_org_required
        CHECK (
            role = 'admin'
            OR (role IN ('agent', 'manager', 'org_admin') AND organization_id IS NOT NULL)
        );

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS chk_users_email_required;

ALTER TABLE users
    ADD CONSTRAINT chk_users_email_required
        CHECK (
            role = 'agent'
            OR (role IN ('admin', 'manager', 'org_admin') AND email IS NOT NULL)
        );

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS chk_users_agent_no_password;

ALTER TABLE users
    ADD CONSTRAINT chk_users_agent_no_password
        CHECK (
            role != 'agent'
            OR (role = 'agent' AND password_hash IS NULL)
        );
