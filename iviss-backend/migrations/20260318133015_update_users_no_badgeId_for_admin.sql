-- Add migration script here
-- =============================================================
-- Migration: 20260317000003_nullable_badge_id.sql
-- Description: Makes badge_id nullable with a constraint
--              that enforces it only for agents.
-- =============================================================

ALTER TABLE users
    ALTER COLUMN badge_id DROP NOT NULL;

ALTER TABLE users
    ADD CONSTRAINT chk_users_agent_badge_id_required
        CHECK (
            role != 'agent'
            OR (role = 'agent' AND badge_id IS NOT NULL)
        );