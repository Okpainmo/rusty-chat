-- Add migration script here
DROP TABLE IF EXISTS messages;

CREATE TABLE messages (
  id BIGSERIAL PRIMARY KEY,
  room_id BIGINT REFERENCES rooms(id) ON DELETE CASCADE,
  sender_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
  type TEXT NOT NULL DEFAULT 'regular',
  content JSONB,
  status TEXT NOT NULL DEFAULT 'sent',
  sent_at VARCHAR(20) NOT NULL,
  edited_at VARCHAR(20),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  CONSTRAINT type_check CHECK (type IN ('regular', 'voice_note', 'voice_call', 'video_call')),
  CONSTRAINT status_check CHECK (status IN ('sent', 'delivered', 'seen'))
);

