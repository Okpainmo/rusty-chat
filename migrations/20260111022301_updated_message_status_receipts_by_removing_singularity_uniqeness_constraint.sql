-- Add migration script here
-- Remove uniqueness constraint to allow multiple receipts per message/user
ALTER TABLE message_status_receipts
DROP CONSTRAINT IF EXISTS message_status_receipts_unique_message_user;

-- Optional: index for history lookups
CREATE INDEX IF NOT EXISTS idx_message_status_receipts_message
ON message_status_receipts (message_id, created_at);
