-- Change pending_submissions image columns to TEXT for base64 storage
ALTER TABLE pending_submissions ALTER COLUMN front_image_url TYPE TEXT;
ALTER TABLE pending_submissions ALTER COLUMN back_image_url TYPE TEXT;
