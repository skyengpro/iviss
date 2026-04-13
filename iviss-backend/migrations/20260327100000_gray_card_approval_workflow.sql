-- Gray Card Approval Workflow
-- Adds approval/rejection columns to pending_submissions
-- Creates submission_audit_log table for tracking admin actions

-- 1. Add review-related columns to pending_submissions
ALTER TABLE pending_submissions
ADD COLUMN IF NOT EXISTS rejection_reason TEXT,
ADD COLUMN IF NOT EXISTS vehicle_data JSONB;

-- reviewed_at already referenced by reviewed_by FK but the timestamp column
-- doesn't exist yet — add it only if missing.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'pending_submissions' AND column_name = 'reviewed_at'
    ) THEN
        ALTER TABLE pending_submissions ADD COLUMN reviewed_at TIMESTAMP;
    END IF;
END
$$;

-- 2. Create submission_audit_log table
CREATE TABLE IF NOT EXISTS submission_audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id UUID NOT NULL REFERENCES pending_submissions(id),
    action VARCHAR(20) NOT NULL,
    performed_by UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    details JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    CONSTRAINT chk_audit_action CHECK (action IN ('approved', 'rejected'))
);

CREATE INDEX IF NOT EXISTS idx_audit_log_submission
    ON submission_audit_log(submission_id);

CREATE INDEX IF NOT EXISTS idx_audit_log_performed_by
    ON submission_audit_log(performed_by);

CREATE INDEX IF NOT EXISTS idx_audit_log_created_at
    ON submission_audit_log(created_at DESC);
