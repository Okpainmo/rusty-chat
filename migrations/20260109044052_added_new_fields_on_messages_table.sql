-- Add migration script here
ALTER TABLE users
ADD COLUMN text_content TEXT,
ADD COLUMN audio_content TEXT,
ADD COLUMN video_content TEXT,
ADD COLUMN images TEXT[]

