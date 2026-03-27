-- Add migration script here
-- =============================================================
-- Migration: 20260317000002_nullable_device_id_refresh_tokens.sql
-- Description: Makes device_id nullable in refresh_tokens.
--              Admin/manager web sessions have no physical device.
-- =============================================================

ALTER TABLE refresh_tokens
    ALTER COLUMN device_id DROP NOT NULL;

ALTER TABLE refresh_tokens
    DROP CONSTRAINT IF EXISTS refresh_tokens_device_id_fkey;

ALTER TABLE refresh_tokens
    ADD CONSTRAINT refresh_tokens_device_id_fkey
        FOREIGN KEY (device_id)
        REFERENCES devices(id)
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED;