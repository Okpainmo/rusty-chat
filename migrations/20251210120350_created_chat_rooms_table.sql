-- Add migration script here
CREATE TABLE rooms (
   id BIGSERIAL PRIMARY KEY,
   room_name TEXT,
   is_group BOOLEAN NOT NULL DEFAULT FALSE,
   created_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
   administrators TEXT[] DEFAULT '{}',
   created_at TIMESTAMP NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
