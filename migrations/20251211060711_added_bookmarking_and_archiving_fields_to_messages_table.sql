-- Add migration script here
ALTER TABLE messages
    ADD COLUMN is_bookmarked BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN is_archived   BOOLEAN NOT NULL DEFAULT FALSE;