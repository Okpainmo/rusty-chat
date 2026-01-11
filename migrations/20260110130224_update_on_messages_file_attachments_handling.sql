-- Add migration script here
ALTER TABLE messages
    DROP COLUMN IF EXISTS audio_content,
    DROP COLUMN IF EXISTS video_content,
    DROP COLUMN IF EXISTS images,
    ADD COLUMN attachment_1 TEXT,
    ADD COLUMN attachment_2 TEXT,
    ADD COLUMN attachment_3 TEXT,
    ADD COLUMN attachment_4 TEXT;
