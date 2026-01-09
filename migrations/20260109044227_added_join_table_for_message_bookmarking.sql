-- Add migration script here
CREATE TABLE message_bookmarks (
    user_id    BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, message_id),

    CONSTRAINT fk_bookmark_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_bookmark_message
        FOREIGN KEY (message_id)
        REFERENCES messages(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_message_bookmarks_user
ON message_bookmarks (user_id);

CREATE INDEX idx_message_bookmarks_message
ON message_bookmarks (message_id);
