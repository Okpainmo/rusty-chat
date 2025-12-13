-- Add migration script here
ALTER TABLE users
    ADD COLUMN is_logged_out BOOLEAN NOT NULL DEFAULT FALSE;
