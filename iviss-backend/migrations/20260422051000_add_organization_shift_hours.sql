ALTER TABLE organizations
    ADD COLUMN shift_start_hour INTEGER NOT NULL DEFAULT 6,
    ADD COLUMN shift_end_hour   INTEGER NOT NULL DEFAULT 18;
