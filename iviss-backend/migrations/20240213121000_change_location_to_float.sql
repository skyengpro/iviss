-- Change location columns to FLOAT8 for easier mapping to f64
ALTER TABLE pending_submissions
ALTER COLUMN latitude TYPE DOUBLE PRECISION,
ALTER COLUMN longitude TYPE DOUBLE PRECISION;
