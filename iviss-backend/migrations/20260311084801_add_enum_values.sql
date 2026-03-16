-- no-transaction
-- Step 1: Add new enum values
-- This must be in a separate migration before using the values

ALTER TYPE device_status ADD VALUE IF NOT EXISTS 'INACTIVE';
ALTER TYPE device_status ADD VALUE IF NOT EXISTS 'SUSPENDED';
ALTER TYPE audit_action ADD VALUE IF NOT EXISTS 'DEVICE_SUSPENDED';
