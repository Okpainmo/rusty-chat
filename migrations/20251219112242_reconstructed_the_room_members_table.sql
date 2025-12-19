-- Add migration script here
DROP TABLE IF EXISTS room_members;

CREATE TABLE room_members (
      id BIGSERIAL PRIMARY KEY,

      room_id BIGINT REFERENCES rooms(id) ON DELETE CASCADE,
      user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,

      role TEXT NOT NULL DEFAULT 'member',
      joined_at VARCHAR(20) NOT NULL,

      created_at TIMESTAMP NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

      CONSTRAINT room_members_unique_room_user UNIQUE (room_id, user_id),
      CONSTRAINT role_check CHECK (role IN ('member', 'admin'))
);
