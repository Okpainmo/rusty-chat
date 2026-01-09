-- Add migration script here
ALTER TABLE users
DROP COLUMN text_content,
DROP COLUMN audio_content,
DROP COLUMN video_content,
DROP COLUMN images;

ALTER TABLE messages
ADD COLUMN text_content TEXT,
ADD COLUMN audio_content TEXT,
ADD COLUMN video_content TEXT,
ADD COLUMN images TEXT[]

