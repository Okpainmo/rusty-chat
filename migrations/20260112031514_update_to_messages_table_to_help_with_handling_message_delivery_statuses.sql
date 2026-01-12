-- Add migration script here
ALTER TABLE messages
ADD COLUMN updates_counter INTEGER NOT NULL DEFAULT 0
CHECK (updates_counter >= 0);

ALTER TABLE messages
DROP CONSTRAINT status_check;

ALTER TABLE messages
ADD CONSTRAINT status_check
CHECK (status IN ('sent', 'delivered', 'seen', 'update'));
