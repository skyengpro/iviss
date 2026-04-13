-- Add constraints for org_admin role
-- Org admins must have an organization_id
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_org_admin_has_org;
ALTER TABLE users ADD CONSTRAINT chk_org_admin_has_org 
    CHECK (role != 'org_admin' OR organization_id IS NOT NULL);
