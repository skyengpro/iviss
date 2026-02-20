-- Add location columns to pending_submissions table
ALTER TABLE pending_submissions
ADD COLUMN latitude DECIMAL(10, 8),
ADD COLUMN longitude DECIMAL(11, 8),
ADD COLUMN address TEXT;
