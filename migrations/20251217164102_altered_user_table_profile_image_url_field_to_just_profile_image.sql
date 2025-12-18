-- Add migration script here
ALTER TABLE users
    RENAME COLUMN profile_image_url TO profile_image;
