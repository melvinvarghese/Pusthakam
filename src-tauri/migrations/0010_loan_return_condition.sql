ALTER TABLE loans ADD COLUMN return_condition TEXT
    CHECK (return_condition IN ('available','damaged','lost'));

UPDATE loans SET return_condition = 'available' WHERE returned_at IS NOT NULL;
