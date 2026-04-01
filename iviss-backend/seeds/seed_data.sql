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
DELETE FROM organizations WHERE id IN (
    '04960f91-57cb-4f23-b308-721998e7373c',
    '17b7eb7c-78a4-43b3-8123-8c3772bff021',
    'be5e79c0-28fc-484e-b66a-435b1c53ee4f'
);

DELETE FROM vehicle_owners WHERE vehicle_id IN (
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    '0690f1ee-6c54-4b01-90e6-d701748f085c',
    '0790f1ee-6c54-4b01-90e6-d701748f085d'
);

DELETE FROM vehicle_statuses WHERE vehicle_id IN (
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    '0690f1ee-6c54-4b01-90e6-d701748f085c',
    '0790f1ee-6c54-4b01-90e6-d701748f085d'
);

DELETE FROM vehicles WHERE id IN (
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    '0690f1ee-6c54-4b01-90e6-d701748f085c',
    '0790f1ee-6c54-4b01-90e6-d701748f085d'
);

-- Delete user-dependent data
DELETE FROM pending_submissions WHERE agent_id IN (SELECT id FROM users WHERE username IN ('manager1', 'manager2', 'agent1', 'agent2'));
DELETE FROM refresh_tokens WHERE user_id IN (SELECT id FROM users WHERE username IN ('manager1', 'manager2', 'agent1', 'agent2'));
DELETE FROM devices WHERE user_id IN (SELECT id FROM users WHERE username IN ('manager1', 'manager2', 'agent1', 'agent2'));
DELETE FROM users WHERE username IN ('manager1', 'manager2', 'agent1', 'agent2');

-- 1. Insert Organization
INSERT INTO organizations (id, name, type, region)
VALUES (
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'National Police Service',
    'police',
    'Capital Region'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO organizations (id, name, type, region)
VALUES 
('04960f91-57cb-4f23-b308-721998e7373c', 'Police Nationale', 'police', 'National'),
('17b7eb7c-78a4-43b3-8123-8c3772bff021', 'Gendarmerie', 'police', 'National'),
('be5e79c0-28fc-484e-b66a-435b1c53ee4f', 'Douanes', 'customs', 'Border')
ON CONFLICT (id) DO NOTHING;

-- 2. Insert Users (updated for new schema with phone_number, status, and nullable email/password_hash)
INSERT INTO users (id, organization_id, username, email, password_hash, role, badge_id, full_name, phone_number, status)
VALUES 
(
    'e390f1ee-6c54-4b01-90e6-d701748f0852',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'manager2',
    'manager2@iviss.gov',
    '$argon2id$v=19$m=19456,t=2,p=1$R7W/+cytB6MIpNljj9mr2w$eaq4+uhWQjiYhlFgd+O/utthO2smslxZAAu7C4Y4yzE',
    'manager',
    'ADM-001',
    'System Administrator',
    '+254700123456',
    'ACTIVE'
),
(
    'f490f1ee-6c54-4b01-90e6-d701748f0853',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'agent1',
    NULL, -- Agents don't need email
    NULL, -- Agents don't have password_hash
    'agent',
    'AGT-102',
    'John Doe',
    '+254700123457',
    'ACTIVE'
),
(
    'e590f1ee-6c54-4b01-90e6-d701748f0854',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'manager1',
    'manager@iviss.gov',
    '$argon2id$v=19$m=19456,t=2,p=1$R7W/+cytB6MIpNljj9mr2w$eaq4+uhWQjiYhlFgd+O/utthO2smslxZAAu7C4Y4yzE',
    'manager',
    'MGR-103',
    'Jane Smith',
    '+254700123458',
    'ACTIVE'
),
(
    'f690f1ee-6c54-4b01-90e6-d701748f0855',
    'd290f1ee-6c54-4b01-90e6-d701748f0851',
    'agent2',
    NULL, -- Agents don't need email
    NULL, -- Agents don't have password_hash
    'agent',
    'AGT-104',
    'Michael Johnson',
    '+237671210292',
    'PENDING_ACTIVATION'
) ON CONFLICT (id) DO NOTHING;

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
    '150 HP'
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
) ,
(
    '0690f1ee-6c54-4b01-90e6-d701748f085c',
    'OU544AU',
    'CHASSIS-OU544AU-001',
    'Peugeot',
    '208',
    2018,
    'Red',
    'Petrol'
) ,
(
    '0790f1ee-6c54-4b01-90e6-d701748f085d',
    'LT128AB',
    'CHASSIS-LT128AB-001',
    'Toyota',
    'Corolla',
    2020,
    'White',
    'Petrol'
) ON CONFLICT (plate_number) DO NOTHING;

-- 4. Vehicle Statuses
INSERT INTO vehicle_statuses (vehicle_id, insurance_status, insurance_expiry, technical_status, technical_expiry, stolen_status, vehicle_image_url)
VALUES 
(
    '0190f1ee-6c54-4b01-90e6-d701748f0854',
    'valid',
    '2025-12-31',
    'valid',
    '2025-06-30',
    FALSE,
    'https://tse1.mm.bing.net/th/id/OIP.Fwr5qO4p1rmMDm2CYwCDZwHaEK?w=326&h=183&c=7&r=0&o=7&cb=defcachec2&pid=1.7&rm=3'
),
(
    '0290f1ee-6c54-4b01-90e6-d701748f0855',
    'expired',
    '2024-01-01',
    'valid',
    '2024-12-31',
    FALSE,
    'https://tse3.mm.bing.net/th/id/OIP.UObfzSWUJy27jyleJS8fXAHaEK?cb=defcachec2&rs=1&pid=ImgDetMain&o=7&rm=3'
),
(
    '0390f1ee-6c54-4b01-90e6-d701748f0856',
    'valid',
    '2026-05-20',
    'valid',
    '2026-01-15',
    FALSE,
    'https://tse1.mm.bing.net/th/id/OIF.PqqzIyivAb9fnDodlDUiwA?cb=defcachec2&rs=1&pid=ImgDetMain&o=7&rm=3'
),
(
    '0490f1ee-6c54-4b01-90e6-d701748f0857',
    'valid',
    '2025-08-15',
    'expired',
    '2023-12-31',
    FALSE,
    'https://tse2.mm.bing.net/th/id/OIP.zj6q8yA5YVogucLo9pSQ4AHaEK?cb=defcachec2&rs=1&pid=ImgDetMain&o=7&rm=3'
),
(
    '0590f1ee-6c54-4b01-90e6-d701748f0858',
    'valid',
    '2025-11-11',
    'valid',
    '2025-11-11',
    TRUE,
    'https://tse2.mm.bing.net/th/id/OIP.tz5wAueaTZtT5H2hXO1fpQHaFj?cb=defcachec2&rs=1&pid=ImgDetMain&o=7&rm=3'

) ,
(
    '0690f1ee-6c54-4b01-90e6-d701748f085c',
    'valid',
    '2026-10-10',
    'valid',
    '2026-10-10',
    FALSE,
    'https://tse4.mm.bing.net/th/id/OIP.qZyWb3QEGxY0NHwGgG_6vQHaEK?cb=defcachec2&rs=1&pid=ImgDetMain&o=7&rm=3'
) ,
(
    '0790f1ee-6c54-4b01-90e6-d701748f085d',
    'valid',
    '2026-12-31',
    'valid',
    '2026-12-31',
    FALSE,
    'https://tse1.mm.bing.net/th/id/OIP.Fwr5qO4p1rmMDm2CYwCDZwHaEK?w=326&h=183&c=7&r=0&o=7&cb=defcachec2&pid=1.7&rm=3'
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
) ,
(
    '0690f1ee-6c54-4b01-90e6-d701748f085c',
    'Aminata Diallo',
    '12 Rue de l''Industrie, Dakar',
    'ID-99887766',
    TRUE
) ,
(
    '0790f1ee-6c54-4b01-90e6-d701748f085d',
    'Paul Nguema',
    'Douala, Littoral Region',
    'ID-1234LT128AB',
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

-- 8. Devices (for agent users - needed for activation, daily login, device management)
INSERT INTO devices (id, user_id, public_key, status, metadata, last_seen_at, created_at)
VALUES 
(
    'd490f1ee-6c54-4b01-90e6-d701748f0853',
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1
    'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtest',
    'ACTIVE',
    '{"os": "Android", "app_version": "1.0.0"}'::jsonb,
    NOW(),
    NOW()
),
(
    'd590f1ee-6c54-4b01-90e6-d701748f0855',
    'f690f1ee-6c54-4b01-90e6-d701748f0855', -- agent2 (INACTIVE - needs activation)
    'MIIBIjANBgkqhkiG9w0BAQEFAA65AQ8AMIIBCgKCAQEAtest',
    'INACTIVE',
    '{}'::jsonb,
    NULL,
    NOW()
),
(
    'd690f1ee-6c54-4b01-90e6-d701748f0856',
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1 second device
    'MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsuspended',
    'SUSPENDED',
    '{"os": "iOS", "app_version": "1.0.0", "suspension_reason": "Test suspension"}'::jsonb,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '30 days'
) ON CONFLICT (id) DO NOTHING;

-- 9. Refresh Tokens (for testing token refresh endpoint)
INSERT INTO refresh_tokens (id, token_hash, user_id, device_id, expires_at, revoked, created_at)
VALUES 
(
    'a190f1ee-6c54-4b01-90e6-d701748f0851',
    'a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789a', -- 64 char hash
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1
    'd490f1ee-6c54-4b01-90e6-d701748f0853',
    NOW() + INTERVAL '30 days',
    FALSE,
    NOW()
),
(
    'a290f1ee-6c54-4b01-90e6-d701748f0852',
    'deadbeefcafebabe0123456789abcdef0123456789abcdef0123456789abcde', -- 64 char hash
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1
    'd690f1ee-6c54-4b01-90e6-d701748f0856', -- suspended device
    NOW() + INTERVAL '30 days',
    TRUE, -- revoked
    NOW() - INTERVAL '7 days'
) ON CONFLICT (id) DO NOTHING;

-- 10. Pending Submissions (for testing admin submission endpoints)
INSERT INTO pending_submissions (id, agent_id, plate_number, front_image_url, back_image_url, notes, status, created_at, updated_at)
VALUES 
(
    'a390f1ee-6c54-4b01-90e6-d701748f0851',
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1
    'NEW 123 PL',
    'https://example.com/front1.jpg',
    'https://example.com/back1.jpg',
    'New vehicle submission',
    'pending',
    NOW() - INTERVAL '2 hours',
    NOW() - INTERVAL '2 hours'
),
(
    'a490f1ee-6c54-4b01-90e6-d701748f0852',
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1
    'OLD 456 QR',
    'https://example.com/front2.jpg',
    'https://example.com/back2.jpg',
    'Old vehicle submission',
    'pending',
    NOW() - INTERVAL '1 day',
    NOW() - INTERVAL '1 day'
),
(
    'a590f1ee-6c54-4b01-90e6-d701748f0853',
    'f490f1ee-6c54-4b01-90e6-d701748f0853', -- agent1
    'LT128AB',
    'https://example.com/front3.jpg',
    'https://example.com/back3.jpg',
    'Existing vehicle submission',
    'pending',
    NOW() - INTERVAL '3 hours',
    NOW() - INTERVAL '3 hours'
) ON CONFLICT (id) DO NOTHING;
