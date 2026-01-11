-- 1. Add sender_id and receiver_id as nullable first
ALTER TABLE message_status_receipts
    DROP COLUMN user_id,
    ADD COLUMN IF NOT EXISTS sender_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS receiver_id BIGINT REFERENCES users(id) ON DELETE CASCADE;

-- 3. Alter sender_id to NOT NULL
ALTER TABLE message_status_receipts
    ALTER COLUMN sender_id SET NOT NULL;

-- 4. Recreate constraints safely
ALTER TABLE message_status_receipts
DROP CONSTRAINT IF EXISTS message_status_check;

ALTER TABLE message_status_receipts
ADD CONSTRAINT message_status_check
    CHECK (status IN ('sent', 'delivered', 'seen'));

ALTER TABLE message_status_receipts
DROP CONSTRAINT IF EXISTS message_status_receipts_action_check;

ALTER TABLE message_status_receipts
ADD CONSTRAINT message_status_receipts_action_check
    CHECK (action IN ('original-send', 'edit', 'delete', 'reaction', 'system'));
