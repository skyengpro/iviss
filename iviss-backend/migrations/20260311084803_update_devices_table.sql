-- Add migration script here
ALTER TABLE devices ADD COLUMN revoked_at TIMESTAMP;
