-- Enhance audit_logs table with IP tracking, resource tracking, and before/after snapshots

-- 1. Add new columns
ALTER TABLE audit_logs
    ADD COLUMN IF NOT EXISTS ip_address    INET,
    ADD COLUMN IF NOT EXISTS resource_type VARCHAR(100),
    ADD COLUMN IF NOT EXISTS resource_id   UUID,
    ADD COLUMN IF NOT EXISTS before_snapshot JSONB,
    ADD COLUMN IF NOT EXISTS after_snapshot  JSONB;

-- 2. Composite index for date-range + action filtering
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created
    ON audit_logs(action, created_at DESC);

-- 3. Index for resource lookups
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource
    ON audit_logs(resource_type, resource_id, created_at DESC)
    WHERE resource_id IS NOT NULL;

-- 4. Immutability trigger — prevent UPDATE and DELETE on audit_logs
CREATE OR REPLACE FUNCTION prevent_audit_log_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit logs are immutable — UPDATE and DELETE operations are not permitted';
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_audit_log_immutability
    BEFORE UPDATE OR DELETE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_log_mutation();
