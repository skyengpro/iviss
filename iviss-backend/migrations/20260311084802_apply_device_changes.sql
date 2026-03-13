-- no-transaction
-- Step 2: Apply device status changes
-- Run this AFTER 20260311084801_add_enum_values.sql

-- Update existing devices to new status values
UPDATE devices 
SET status = 'INACTIVE' 
WHERE status = 'PENDING';

UPDATE devices 
SET status = 'SUSPENDED' 
WHERE status = 'REVOKED';

-- Drop and recreate the index with updated filter
DROP INDEX IF EXISTS idx_devices_user_id;
CREATE INDEX idx_devices_user_id
    ON devices(user_id)
    WHERE status != 'SUSPENDED';

-- Rename revoked_at to suspended_at for clarity
ALTER TABLE devices RENAME COLUMN revoked_at TO suspended_at;
