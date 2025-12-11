-- Add migration script here
ALTER TABLE users
    ADD COLUMN status VARCHAR(10) NOT NULL DEFAULT 'offline';

ALTER TABLE users
    ADD COLUMN last_seen VARCHAR(20);
