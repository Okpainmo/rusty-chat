-- Add migration script here
ALTER TABLE users
    ADD COLUMN access_token VARCHAR(1024),
    ADD COLUMN refresh_token VARCHAR(1024),
    ADD COLUMN one_time_password_token VARCHAR(1024);
