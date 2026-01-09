-- Add migration script here
ALTER TABLE messages
DROP COLUMN IF EXISTS content
