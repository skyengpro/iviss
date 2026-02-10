-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 1. Organizations
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    region VARCHAR(100),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP NULL,
    CONSTRAINT chk_org_type CHECK (type IN ('police', 'customs', 'border_control', 'other'))
);

CREATE TRIGGER update_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 2. Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL,
    role VARCHAR(20) NOT NULL,
    badge_id VARCHAR(50),
    full_name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP NULL,
    CONSTRAINT chk_user_role CHECK (role IN ('admin', 'agent', 'manager'))
);

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE INDEX idx_users_organization_role ON users(organization_id, role) WHERE is_active = TRUE;
CREATE INDEX idx_users_email ON users(email) WHERE is_active = TRUE;

-- 3. Vehicles
CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plate_number VARCHAR(20) UNIQUE NOT NULL,
    chassis_number VARCHAR(50) UNIQUE,
    brand VARCHAR(50),
    model VARCHAR(50),
    year INTEGER,
    color VARCHAR(30),
    engine_power VARCHAR(20),
    fuel_type VARCHAR(20),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP NULL
);

CREATE TRIGGER update_vehicles_updated_at
    BEFORE UPDATE ON vehicles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE INDEX idx_vehicles_chassis ON vehicles(chassis_number) WHERE chassis_number IS NOT NULL;
CREATE INDEX idx_vehicles_plate_number ON vehicles(plate_number); -- Implicitly created by UNIQUE constraint, but good to be explicit or rely on constraint

-- 4. Vehicle Owners
CREATE TABLE vehicle_owners (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id),
    name VARCHAR(255) NOT NULL,
    address TEXT,
    national_id VARCHAR(50),
    ownership_start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    ownership_end_date DATE NULL,
    is_current_owner BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP NULL
);

CREATE TRIGGER update_vehicle_owners_updated_at
    BEFORE UPDATE ON vehicle_owners
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 5. Vehicle Statuses
CREATE TABLE vehicle_statuses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id UUID UNIQUE NOT NULL REFERENCES vehicles(id),
    insurance_status VARCHAR(20),
    insurance_expiry DATE,
    technical_status VARCHAR(20),
    technical_expiry DATE,
    stolen_status BOOLEAN DEFAULT FALSE,
    last_updated TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    CONSTRAINT chk_insurance_status CHECK (insurance_status IN ('valid', 'expired', 'none', 'unknown')),
    CONSTRAINT chk_technical_status CHECK (technical_status IN ('valid', 'expired', 'failed', 'unknown'))
);

CREATE TRIGGER update_vehicle_statuses_updated_at
    BEFORE UPDATE ON vehicle_statuses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE INDEX idx_vehicle_statuses_expired_insurance ON vehicle_statuses(insurance_expiry) WHERE insurance_status = 'expired';

-- 6. Control Records
CREATE TABLE control_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES users(id),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    plate_number VARCHAR(20) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    address TEXT,
    identification_mode VARCHAR(20),
    ocr_confidence INTEGER,
    overall_status VARCHAR(20),
    results_json JSONB,
    notes TEXT,
    vehicle_id UUID NULL REFERENCES vehicles(id),
    photo_url VARCHAR(255) NULL,
    device_id VARCHAR(100) NULL,
    app_version VARCHAR(20) NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP NULL,
    CONSTRAINT chk_identification_mode CHECK (identification_mode IN ('manual', 'photo', 'live')),
    CONSTRAINT chk_overall_status CHECK (overall_status IN ('valid', 'warning', 'critical')),
    CONSTRAINT chk_ocr_confidence CHECK (ocr_confidence >= 0 AND ocr_confidence <= 100)
);

CREATE TRIGGER update_control_records_updated_at
    BEFORE UPDATE ON control_records
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE INDEX idx_control_records_timestamp ON control_records(timestamp DESC);
CREATE INDEX idx_control_records_org_timestamp ON control_records(organization_id, timestamp DESC);
CREATE INDEX idx_control_records_status ON control_records(overall_status);
CREATE INDEX idx_control_records_vehicle ON control_records(vehicle_id) WHERE vehicle_id IS NOT NULL;
CREATE INDEX idx_control_records_plate ON control_records(plate_number);
CREATE INDEX idx_control_records_results_json ON control_records USING GIN (results_json);

-- 7. Control Actions
CREATE TABLE control_actions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    control_id UUID NOT NULL REFERENCES control_records(id),
    action_type VARCHAR(50) NOT NULL,
    description TEXT,
    timestamp TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    CONSTRAINT chk_action_type CHECK (action_type IN ('citation', 'impound', 'flag', 'warning'))
);

CREATE TRIGGER update_control_actions_updated_at
    BEFORE UPDATE ON control_actions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 8. Pending Submissions
CREATE TABLE pending_submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES users(id),
    plate_number VARCHAR(20) NOT NULL,
    front_image_url VARCHAR(255),
    back_image_url VARCHAR(255),
    notes TEXT,
    status VARCHAR(20) DEFAULT 'pending',
    reviewed_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    CONSTRAINT chk_submission_status CHECK (status IN ('pending', 'approved', 'rejected'))
);

CREATE TRIGGER update_pending_submissions_updated_at
    BEFORE UPDATE ON pending_submissions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE INDEX idx_pending_submissions_status ON pending_submissions(status, created_at DESC) WHERE status = 'pending';
