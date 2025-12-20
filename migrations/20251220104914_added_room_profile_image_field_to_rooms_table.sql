-- Add migration script here
-- Add a new column 'room_profile_image' to store the room's profile image URL
ALTER TABLE rooms
    ADD COLUMN room_profile_image TEXT DEFAULT '';

-- Optional: add a comment for clarity
COMMENT ON COLUMN rooms.room_profile_image IS 'URL or path of the room profile image';
