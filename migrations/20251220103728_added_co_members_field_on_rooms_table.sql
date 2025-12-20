-- Add migration script here
ALTER TABLE rooms
    ADD COLUMN co_members BIGINT[] DEFAULT '{}';

-- Optional: add a comment
COMMENT ON COLUMN rooms.co_members IS 'Array of user IDs for members of the room';
