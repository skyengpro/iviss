-- no-transaction
-- Add INACTIVE to device_status enum for shift expiry handling
ALTER TYPE device_status ADD VALUE IF NOT EXISTS 'INACTIVE';
