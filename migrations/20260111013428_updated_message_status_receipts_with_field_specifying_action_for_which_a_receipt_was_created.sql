-- 1. Add column WITH DEFAULT so existing rows are populated
ALTER TABLE message_status_receipts
ADD COLUMN action TEXT DEFAULT 'original-send';

-- 2. Backfill explicitly (good practice, even with default)
UPDATE message_status_receipts
SET action = 'original-send'
WHERE action IS NULL;

-- 3. Enforce NOT NULL + constraint
ALTER TABLE message_status_receipts
ALTER COLUMN action SET NOT NULL;

ALTER TABLE message_status_receipts
ADD CONSTRAINT message_status_receipts_action_check
CHECK (
    action IN (
        'original-send',
        'edit',
        'delete',
        'reaction',
        'system'
    )
);

-- 4. (Optional but recommended) Remove default
ALTER TABLE message_status_receipts
ALTER COLUMN action DROP DEFAULT;

-- 5. Index for performance
CREATE INDEX idx_message_status_receipts_action
ON message_status_receipts (action);
