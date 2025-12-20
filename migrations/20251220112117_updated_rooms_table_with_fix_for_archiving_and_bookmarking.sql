-- Add migration script here
-- Remove old boolean columns from rooms table
ALTER TABLE rooms DROP COLUMN IF EXISTS is_bookmarked;
ALTER TABLE rooms DROP COLUMN IF EXISTS is_archived;

-- Add new many-to-one relationship columns
-- These will store arrays of user IDs who have bookmarked/archived the room

ALTER TABLE rooms
    ADD COLUMN bookmarked_by BIGINT[] DEFAULT '{}',
    ADD COLUMN archived_by BIGINT[] DEFAULT '{}';

-- Create indexes for array columns to improve query performance
CREATE INDEX idx_rooms_bookmarked_by ON rooms USING GIN(bookmarked_by);
CREATE INDEX idx_rooms_archived_by ON rooms USING GIN(archived_by);