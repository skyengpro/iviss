-- Seed Data for IVISS Database

-- Cleanup (to ensure no duplicate seed data with old formats)
DELETE FROM control_actions WHERE control_id IN (
    'a190f1ee-6c54-4b01-90e6-d701748f0859',
    'a290f1ee-6c54-4b01-90e6-d701748f0860',
    'a390f1ee-6c54-4b01-90e6-d701748f0861'
);

DELETE FROM control_records WHERE id IN (
    'a190f1ee-6c54-4b01-90e6-d701748f0859',
    'a290f1ee-6c54-4b01-90e6-d701748f0860',
    'a390f1ee-6c54-4b01-90e6-d701748f0861'
);

DELETE FROM vehicle_owners WHERE vehicle_id IN (
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    '0590f1ee-6c54-4b01-90e6-d701748f0858'
);

DELETE FROM vehicle_statuses WHERE vehicle_id IN (
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    '0590f1ee-6c54-4b01-90e6-d701748f0858'
);

DELETE FROM vehicles WHERE id IN (
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    '0590f1ee-6c54-4b01-90e6-d701748f0858'
);

-- 1. Insert Organization
INSERT INTO organizations (id, name, type, region)
VALUES (
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'National Police Service',
    'police',
    'Capital Region'
) ON CONFLICT (id) DO NOTHING;

-- 2. Insert Users
INSERT INTO users (id, organization_id, username, email, password_hash, role, badge_id, full_name)
VALUES 
(
    'e390f1ee-6c54-4b01-90e6-d701748f0852',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'admin',
    'admin@iviss.gov',
    '$2b$12$LQv3c1yqBWVHxkd0LqZGueOQ/H/XJmK8m7B/K8yK8yK8yK8yK8yK8',
    'admin',
    'ADM-001',
    'System Administrator'
),
(
    'f490f1ee-6c54-4b01-90e6-d701748f0853',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'agent1',
    'agent1@iviss.gov',
    '$2b$12$LQv3c1yqBWVHxkd0LqZGueOQ/H/XJmK8m7B/K8yK8yK8yK8yK8yK8',
    'agent',
    'AGT-102',
    'John Doe'
) ON CONFLICT (username) DO NOTHING;

-- 3. Insert Vehicles (5 diverse plates in format: AD 345 CE)
INSERT INTO vehicles (id, plate_number, chassis_number, brand, model, year, color, fuel_type)
VALUES 
(
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    'AD 345 CE',
    'CHASSIS-HILUX-001',
    'Toyota',
    'Hilux',
    2022,
    'White',
    'Diesel'
),
(
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    'BC 123 DF',
    'CHASSIS-DMAX-123',
    'Isuzu',
    'D-Max',
    2021,
    'Silver',
    'Diesel'
),
(
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    'GH 789 JK',
    'CHASSIS-LANDCRUISER-789',
    'Toyota',
    'Land Cruiser',
    2023,
    'Black',
    'Petrol'
),
(
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    'LM 456 NP',
    'CHASSIS-GOLF-456',
    'Volkswagen',
    'Golf',
    2020,
    'Blue',
    'Petrol'
),
(
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    'RS 999 TV',
    'CHASSIS-ECLASS-999',
    'Mercedes-Benz',
    'E-Class',
    2019,
    'Grey',
    'Petrol'
) ON CONFLICT (plate_number) DO NOTHING;

-- 4. Vehicle Statuses
INSERT INTO vehicle_statuses (vehicle_id, insurance_status, insurance_expiry, technical_status, technical_expiry, stolen_status)
VALUES 
(
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    'valid',
    '2025-12-31',
    'valid',
    '2025-06-30',
    FALSE
),
(
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    'expired',
    '2024-01-01',
    'valid',
    '2024-12-31',
    FALSE
),
(
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    'valid',
    '2026-05-20',
    'valid',
    '2026-01-15',
    FALSE
),
(
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    'valid',
    '2025-08-15',
    'expired',
    '2023-12-31',
    FALSE
),
(
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    'valid',
    '2025-11-11',
    'valid',
    '2025-11-11',
    TRUE
) ON CONFLICT (vehicle_id) DO NOTHING;

-- 5. Vehicle Owners
INSERT INTO vehicle_owners (vehicle_id, name, address, national_id, is_current_owner)
VALUES 
(
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    'Peter Kamau',
    '123 Nairobi St, Nairobi',
    'ID-12345678',
    TRUE
),
(
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    'Jane Anyango',
    '456 Kisumu Rd, Kisumu',
    'ID-87654321',
    TRUE
),
(
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    'Abdullah Hassan',
    '789 Mombasa Ave, Mombasa',
    'ID-11223344',
    TRUE
),
(
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    'Mary Wanjiku',
    '321 Nakuru Way, Nakuru',
    'ID-44332211',
    TRUE
),
(
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    'David Omondi',
    '654 Eldoret St, Eldoret',
    'ID-55667788',
    TRUE
) ON CONFLICT DO NOTHING;

-- 6. Control Records
INSERT INTO control_records (id, agent_id, organization_id, plate_number, timestamp, latitude, longitude, address, identification_mode, ocr_confidence, overall_status, vehicle_id)
VALUES 
(
    'a190f1ee-6c54-4b01-90e6-d701748f0859',
    'f490f1ee-6c54-4b01-90e6-d701748f0853',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'AD 345 CE',
    NOW() - INTERVAL '2 hours',
    -1.283333,
    36.816667,
    'City Center, Nairobi',
    'photo',
    98,
    'valid',
    '0190f1ee-6c54-4b01-90e6-d701748f0854'
),
(
    'a290f1ee-6c54-4b01-90e6-d701748f0860',
    'f490f1ee-6c54-4b01-90e6-d701748f0853',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'BC 123 DF',
    NOW() - INTERVAL '1 day',
    -1.300000,
    36.783333,
    'Westlands, Nairobi',
    'manual',
    100,
    'warning',
    '0290f1ee-6c54-4b01-90e6-d701748f0855'
),
(
    'a390f1ee-6c54-4b01-90e6-d701748f0861',
    'f490f1ee-6c54-4b01-90e6-d701748f0853',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'RS 999 TV',
    NOW() - INTERVAL '5 hours',
    -1.250000,
    36.900000,
    'Thika Road, Nairobi',
    'live',
    92,
    'critical',
    '0590f1ee-6c54-4b01-90e6-d701748f0858'
) ON CONFLICT (id) DO NOTHING;

-- 7. Control Actions
INSERT INTO control_actions (control_id, action_type, description)
VALUES 
(
    'a290f1ee-6c54-4b01-90e6-d701748f0860',
    'warning',
    'Insurance expired. Driver notified.'
),
(
    'a390f1ee-6c54-4b01-90e6-d701748f0861',
    'impound',
    'Vehicle reported stolen. Local authorities alerted.'
) ON CONFLICT DO NOTHING;
