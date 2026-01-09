-- Add migration script here

-- Add new many-to-one relationship columns
-- These will store arrays of user IDs who have pinned the room

ALTER TABLE rooms
    ADD COLUMN pinned_by BIGINT[] DEFAULT '{}';

-- Create indexes for array columns to improve query performance
CREATE INDEX idx_rooms_pinned_by ON rooms USING GIN(pinned_by);