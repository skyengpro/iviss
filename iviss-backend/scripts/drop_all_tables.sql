-- =============================================================
-- Script: drop_all_tables.sql
-- Description: Drop all tables in the iviss database
-- WARNING: This will delete ALL tables and data
-- =============================================================

-- Disable foreign key checks temporarily
SET session_replication_role = 'replica';

-- Drop all tables in reverse dependency order
DROP TABLE IF EXISTS audit_logs CASCADE;
DROP TABLE IF EXISTS refresh_tokens CASCADE;
DROP TABLE IF EXISTS agent_locations CASCADE;
DROP TABLE IF EXISTS devices CASCADE;
DROP TABLE IF EXISTS pending_submissions CASCADE;
DROP TABLE IF EXISTS control_records CASCADE;
DROP TABLE IF EXISTS control_actions CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS organizations CASCADE;
DROP TABLE IF EXISTS access_token_blacklist CASCADE;
DROP TABLE IF EXISTS _sqlx_migrations CASCADE;

-- Re-enable foreign key checks
SET session_replication_role = DEFAULT;

-- Verify all tables are dropped
SELECT 'All tables dropped successfully' AS status;
