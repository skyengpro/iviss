-- Add migration script here

CREATE TYPE device_status AS ENUM (
    'PENDING',
    'ACTIVE',
    'REVOKED'
);

CREATE TABLE devices (
    id              UUID            PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID            NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key      TEXT            NOT NULL,
    metadata        JSONB           NOT NULL DEFAULT '{}',
    status          device_status   NOT NULL DEFAULT 'PENDING',
    last_seen_at    TIMESTAMP,                     
    created_at      TIMESTAMP       NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP       NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMP                      
);

CREATE TRIGGER update_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ─────────────────────────────────────────
-- Index
-- ─────────────────────────────────────────

CREATE INDEX idx_devices_user_id
    ON devices(user_id)
    WHERE status != 'REVOKED';

CREATE UNIQUE INDEX idx_devices_public_key
    ON devices(public_key);


-- ─────────────────────────────────────────
--  Refresh Tokens
-- ─────────────────────────────────────────
CREATE TABLE refresh_tokens (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    token_hash      VARCHAR(64) NOT NULL UNIQUE,
    user_id         UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id       UUID        NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    expires_at      TIMESTAMP   NOT NULL,
    revoked         BOOLEAN     NOT NULL DEFAULT FALSE,
    revoked_at      TIMESTAMP,                      -- audit trail
    created_at      TIMESTAMP   NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────
-- Index
-- ─────────────────────────────────────────

CREATE INDEX idx_refresh_tokens_token_hash
    ON refresh_tokens(token_hash)
    WHERE revoked = FALSE;

-- Cleanup expired tokens
CREATE INDEX idx_refresh_tokens_user_id
    ON refresh_tokens(user_id, expires_at)
    WHERE revoked = FALSE;

CREATE INDEX idx_refresh_tokens_expires_at
    ON refresh_tokens(expires_at)
    WHERE revoked = FALSE;

-- ─────────────────────────────────────────
--  Access Token Blacklist
-- ─────────────────────────────────────────
CREATE TABLE access_token_blacklist (
    jti             VARCHAR(36) PRIMARY KEY,
    user_id         UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at      TIMESTAMP   NOT NULL,
    created_at      TIMESTAMP   NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blacklist_expires_at
    ON access_token_blacklist(expires_at);


-- ─────────────────────────────────────────
-- audit actions ENUM 
-- ─────────────────────────────────────────
CREATE TYPE audit_action AS ENUM (
    -- Auth
    'LOGIN_SUCCESS',
    'LOGIN_FAILED',
    'LOGOUT',
    'TOKEN_REFRESHED',
    'OTP_REQUESTED',
    'OTP_VERIFIED',
    'OTP_FAILED',

    -- Device
    'DEVICE_REGISTERED',
    'DEVICE_REVOKED',

    -- User management
    'USER_CREATED',
    'USER_UPDATED',
    'USER_SUSPENDED',
    'USER_ACTIVATED',
    'USER_DELETED',
    'SESSION_TERMINATED',
    'SESSION_RESTARTED',
    'ACTIVATION_CODE_RESENT',

    -- Vehicle control
    'VEHICLE_SEARCHED',
    'VEHICLE_NOT_FOUND',
    'PENDING_SUBMISSION_CREATED',
    'PENDING_SUBMISSION_REVIEWED'
);

-- ─────────────────────────────────────────
-- audit_logs Table
-- ─────────────────────────────────────────
CREATE TABLE audit_logs (
    id          UUID            PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID            REFERENCES users(id) ON DELETE SET NULL,
    device_id   UUID            REFERENCES devices(id) ON DELETE SET NULL,
    action      audit_action    NOT NULL,
    metadata    JSONB           NOT NULL DEFAULT '{}',
    created_at  TIMESTAMP       NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────
--  Index
-- ─────────────────────────────────────────

CREATE INDEX idx_audit_logs_user_id
    ON audit_logs(user_id, created_at DESC)
    WHERE user_id IS NOT NULL;

CREATE INDEX idx_audit_logs_device_id
    ON audit_logs(device_id, created_at DESC)
    WHERE device_id IS NOT NULL;


CREATE INDEX idx_audit_logs_action
    ON audit_logs(action, created_at DESC);

CREATE INDEX idx_audit_logs_created_at
    ON audit_logs(created_at DESC);

CREATE INDEX idx_audit_logs_metadata
    ON audit_logs USING GIN(metadata);