-- Remove administrators column from rooms table
ALTER TABLE rooms
    DROP COLUMN IF EXISTS administrators;
