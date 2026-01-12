-- Add migration script here
CREATE TABLE IF NOT EXISTS message_reactions (
    id BIGSERIAL PRIMARY KEY,

    message_id BIGINT NOT NULL
        REFERENCES messages(id) ON DELETE CASCADE,

    room_id BIGINT NOT NULL
        REFERENCES rooms(id) ON DELETE CASCADE,

    sender_id BIGINT NOT NULL
        REFERENCES users(id) ON DELETE CASCADE,

    reaction_type TEXT NOT NULL,

    -- tracks which message version the reaction applies to
    message_updates_counter INTEGER NOT NULL DEFAULT 0
        CHECK (message_updates_counter >= 0),

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- one reaction type per user per message
    CONSTRAINT unique_user_reaction
        UNIQUE (message_id, sender_id, reaction_type)
);

-- fast reaction lookup per message
CREATE INDEX idx_message_reactions_message
ON message_reactions (message_id);

-- fast room-wide aggregation (group chats)
CREATE INDEX idx_message_reactions_room
ON message_reactions (room_id);

-- fast per-user reaction management
CREATE INDEX idx_message_reactions_sender
ON message_reactions (sender_id);
