-- Add migration script here
CREATE TABLE message_status_receipts (
     message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
     user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
     room_id    BIGINT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,

     status TEXT NOT NULL,

     created_at TIMESTAMP NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

     CONSTRAINT message_status_receipts_pk
         PRIMARY KEY (message_id, user_id),

     CONSTRAINT message_status_check
         CHECK (status IN ('sent', 'delivered', 'seen'))
);
