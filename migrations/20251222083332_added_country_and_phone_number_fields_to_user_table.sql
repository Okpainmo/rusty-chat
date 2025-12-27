-- Add migration script here
ALTER TABLE users
    ADD COLUMN phone_number VARCHAR(20) UNIQUE,
    ADD COLUMN country VARCHAR(100);
