-- Add migration script here
ALTER TABLE message_status_receipts
ADD COLUMN updates_count_tracker INTEGER NOT NULL DEFAULT 0
CHECK (updates_count_tracker >= 0);

CREATE INDEX idx_message_receipts_update_tracking
ON message_status_receipts (message_id, updates_count_tracker);
