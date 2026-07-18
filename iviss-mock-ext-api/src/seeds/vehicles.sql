-- Seed data for iviss-test-api mock vehicle database.
-- Applied on startup with INSERT ... ON CONFLICT DO NOTHING, so safe to re-run.
-- All plate_number values stored with spaces and UPPERCASE (normalised on insert).
-- Covers: NOT_CLEARED/CLEARED mix, missing optional fields, varied mark_and_type formats.

INSERT INTO test_api.vehicles
    (plate_number, chassis_number, mark_and_type, engine_power, owner_name, nps_status, customs_status)
VALUES
    -- ── CE prefix ────────────────────────────────────────────────────────────
    ('CE 568 LR',  'WDB4632341X258849', 'MERCEDES AB54E2',         '19', 'VESSAH MOHAMED',      'RAS',     'NOT_CLEARED'),
    ('CE 112 AB',  'VF7RD8HZB87654321', 'PEUGEOT 605 6BR2A29',    '15', 'DUPONT JEAN',          'RAS',     'CLEARED'),
    ('CE 301 MK',  'JTDBF12E200001234', 'TOYOTA KG9630',           '12', 'ABANDA PIERRE',        'RAS',     'CLEARED'),
    ('CE 044 XZ',  NULL,               'BMW HB 11',                '20', 'NKOLO SYLVIE',         'RAS',     'NOT_CLEARED'),
    ('CE 789 PQ',  'WVWZZZ1JZXW123456', 'Opel Manta',             '11', 'FOUDA CELESTIN',       'PENDING', 'CLEARED'),
    ('CE 200 TK',  'SCC217502CHR12345', NULL,                      '18', 'MBARGA ROSE',          'RAS',     'NOT_CLEARED'),
    ('CE 555 RR',  'JF1GG71S47G789012', 'Toyota LST 771',          '16', NULL,                   'RAS',     'CLEARED'),
    ('CE 091 YY',  'KMHCM41BP5A012345', 'MITSUBISHI A172',         '14', 'ETOUNDI PAUL',         'RAS',     'NOT_CLEARED'),
    ('CE 777 ZA',  'ZFA18800000123456', 'FIAT BRAVO',              NULL, 'OBAMA CHRISTIANE',     'RAS',     'CLEARED'),
    ('CE 003 LL',  'WBA3A5G58DNS00001', 'BMW',                     '22', 'BEYEM ALPHONSE',       'RAS',     'NOT_CLEARED'),

    -- ── LT prefix ────────────────────────────────────────────────────────────
    ('LT 045 AA',  'SB1GP56U07E012345', 'Toyota',                  '13', 'NGUENA MARC',          'RAS',     'CLEARED'),
    ('LT 128 BC',  'WDD2050421R012345', 'MERCEDES 180',            '17', 'ATANGANA HELENE',      'RAS',     'NOT_CLEARED'),
    ('LTSR 307 DE',  'JN1BKAJ11U012345',  'NISSAN MICRA',            '10', 'MVONDO ETIENNE',       'RAS',     'CLEARED'),
    ('LT 491 FG',  NULL,               'Opel',                     '12', 'NKENGNE SUZANNE',      'RAS',     'CLEARED'),
    ('LT 600 HJ',  'WVWDB4505LK012345', 'VOLKSWAGEN POLO',         '15', 'BIKOE ROGER',          'RAS',     'NOT_CLEARED'),
    ('LT 050 KL',  'TW00R1MGDH4012345', NULL,                      '11', NULL,                   'RAS',     'CLEARED'),

    -- ── SN and SU prefix ────────────────────────────────────────────────────────────
    ('SN 490 MN',  'KMHDN41BP4A012345', 'HYUNDAI ACCENT',          '14', 'ONANA JEAN-CLAUDE',    'RAS',     'CLEARED'),
    ('SU 022 OP',  'VF1BA000529012345', 'RENAULT CLIO',            '10', 'ESSONO MARIE',         'RAS',     'NOT_CLEARED'),
    ('SN 815 QR',  'ZAHBR7AG9F4012345', 'Peugeot',                 '16', 'BIYONG GABRIEL',       'RAS',     'CLEARED'),
    ('SNRE 815 QR','HACBR7AG9F4012345', 'MERCEDES BENZ C200',      '16', 'SOCIETE DES AUTO MOTEURS',       'RAS',     'CLEARED'),
    ('SU 134 ST',  NULL,               'TOYOTA 53SBN0',            '13', 'ATEMENGUE LUCIE',      'RAS',     'NOT_CLEARED'),

    -- ── EN prefix ────────────────────────────────────────────────────────────
    ('EN 210 UV',  'WDB9636321L012345', 'MERCEDES 180',            '19', 'MINYEM FRANCOIS',      'RAS',     'CLEARED'),
    ('EN 088 WX',  'VF1BB000512012345', 'RENAULT MEGANE',          '14', NULL,                   'RAS',     'NOT_CLEARED'),
    ('EN 500 YZ',  'KNAFE121795012345', NULL,                       NULL, 'OWONO BERNADETTE',     'RAS',     'CLEARED'),

    -- ── NO prefix ────────────────────────────────────────────────────────────
    ('NO 331 AK',  'JSAAZS21S00012345', 'SUZUKI',                  '10', 'NKEMDIRIM SUNDAY',     'RAS',     'NOT_CLEARED'),
    ('NO 042 VB',  'SHH0P37021U012345', 'HONDA CIVIC',             '12', 'NGOUMOU FELIX',        'RAS',     'CLEARED'),
    ('SW 652 CB',  'SHH0P37021U012345', 'HONDA  LV32',             '32', 'ALIN FELIX',        'RAS',     'CLEARED'),
    ('NO 199 DC',  NULL,               'Toyota',                   '13', 'ABENA PATRICIA',       'RAS',     'CLEARED'),

    -- ── Edge cases ───────────────────────────────────────────────────────────
    -- All optional fields NULL (only plate_number present — parser should handle gracefully)
    ('CE 999 ED',  NULL,               NULL,                        NULL, NULL,                   NULL,      'CLEARED'),
    -- Very long mark_and_type
    ('LT 001 XL',  'WDB9104321R099999', 'MERCEDES BENZ SPRINTER 316 CDI LONG',
                                                                    '19', 'TAGNE LEOPOLD',        'RAS',     'NOT_CLEARED'),
    -- customs_status value that is neither CLEARED nor NOT_CLEARED
    ('SN 777 RX',  'VF7RC8HNB00099999', 'PEUGEOT 308',             '15', 'BASSONG SERGE',        'RAS',     'SOUS_DOUANE')

ON CONFLICT DO NOTHING;
