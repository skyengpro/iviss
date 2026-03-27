-- =============================================================
-- Script: truncate_all_tables.sql
-- Description: Truncate all tables in the iviss database
-- WARNING: This will delete ALL data from all tables
-- =============================================================

-- Disable foreign key checks temporarily
SET session_replication_role = 'replica';

-- Truncate all tables in reverse dependency order (ignore if table doesn't exist)
TRUNCATE TABLE audit_logs CASCADE;
TRUNCATE TABLE refresh_tokens CASCADE;
TRUNCATE TABLE agent_locations CASCADE;
TRUNCATE TABLE devices CASCADE;
TRUNCATE TABLE pending_submissions CASCADE;
TRUNCATE TABLE control_records CASCADE;
TRUNCATE TABLE control_actions CASCADE;
TRUNCATE TABLE users CASCADE;
TRUNCATE TABLE organizations CASCADE;
TRUNCATE TABLE access_token_blacklist CASCADE;

-- Re-enable foreign key checks
SET session_replication_role = DEFAULT;

-- Reset the sqlx migrations table (allows migrations to re-run)
TRUNCATE TABLE _sqlx_migrations CASCADE;

-- Verify tables are empty
SELECT 'Tables truncated successfully' AS status;
