-- Add migration script here
ALTER TABLE messages
DROP CONSTRAINT status_check;

ALTER TABLE messages
ADD CONSTRAINT status_check
CHECK (status IN ('sent', 'delivered', 'seen', 'updated'));
