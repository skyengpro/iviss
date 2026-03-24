-- =============================================================
-- Migration: 20260311084806_add_device_suspension_index.sql
-- Description: Add index for device suspension queries
-- =============================================================

-- Add index on device suspension-related columns for query performance
CREATE INDEX IF NOT EXISTS idx_devices_suspended ON devices (revoked_at) WHERE revoked_at IS NOT NULL;
