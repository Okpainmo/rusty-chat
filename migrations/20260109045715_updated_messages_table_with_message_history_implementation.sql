-- Add migration script here

-- Remove old edited_at flag
ALTER TABLE messages
DROP COLUMN IF EXISTS edited_at;

-- Track message edit history
CREATE TABLE IF NOT EXISTS message_edits (
    id BIGSERIAL PRIMARY KEY,

    message_id BIGINT NOT NULL
        REFERENCES messages(id)
        ON DELETE CASCADE,

    previous_context TEXT NOT NULL,
    new_content TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_edits_message_id
ON message_edits (message_id);

CREATE INDEX IF NOT EXISTS idx_message_edits_created_at
ON message_edits (created_at);
