-- Add migration script here
CREATE TABLE message_archives (
    user_id    BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, message_id),

    CONSTRAINT fk_archive_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_archive_message
        FOREIGN KEY (message_id)
        REFERENCES messages(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_message_archives_user
ON message_archives (user_id);

CREATE INDEX idx_message_archives_message
ON message_archives (message_id);