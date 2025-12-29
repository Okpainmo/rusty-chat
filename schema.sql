-- consolidated schema for the rusty-chat project
-- this file contains the create statements for all tables in the database

-- Users Table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL, -- Stores Argon2 hashed password
    full_name VARCHAR(511) NOT NULL, -- Computed from first_name + last_name
    profile_image VARCHAR(512),
    access_token VARCHAR(1024),
    refresh_token VARCHAR(1024),
    one_time_password_token VARCHAR(1024),
    status VARCHAR(10) NOT NULL DEFAULT 'offline',
    last_seen VARCHAR(20),
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_logged_out BOOLEAN NOT NULL DEFAULT FALSE,
    phone_number VARCHAR(20) UNIQUE,
    country VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Index for users.email
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Rooms Table
CREATE TABLE IF NOT EXISTS rooms (
    id BIGSERIAL PRIMARY KEY,
    room_name TEXT,
    is_group BOOLEAN NOT NULL DEFAULT FALSE,
    created_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    co_member BIGINT UNIQUE, -- for private-chats only
    co_members BIGINT[] DEFAULT '{}',
    room_profile_image TEXT DEFAULT '',
    bookmarked_by BIGINT[] DEFAULT '{}',
    archived_by BIGINT[] DEFAULT '{}',
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_co_member FOREIGN KEY (co_member) REFERENCES users(id)
);

-- Comments for rooms columns
COMMENT ON COLUMN rooms.co_members IS 'Array of user IDs for members of the room';
COMMENT ON COLUMN rooms.room_profile_image IS 'URL or path of the room profile image';

-- GIN indexes for rooms array columns
CREATE INDEX IF NOT EXISTS idx_rooms_bookmarked_by ON rooms USING GIN(bookmarked_by);
CREATE INDEX IF NOT EXISTS idx_rooms_archived_by ON rooms USING GIN(archived_by);

-- Room Members Table
CREATE TABLE IF NOT EXISTS room_members (
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

-- Messages Table
CREATE TABLE IF NOT EXISTS messages (
  id BIGSERIAL PRIMARY KEY,
  room_id BIGINT REFERENCES rooms(id) ON DELETE CASCADE,
  sender_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
  type TEXT NOT NULL DEFAULT 'regular',
  content JSONB,
  status TEXT NOT NULL DEFAULT 'sent',
  sent_at VARCHAR(20) NOT NULL,
  edited_at VARCHAR(20),
  is_bookmarked BOOLEAN NOT NULL DEFAULT FALSE,
  is_archived BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  CONSTRAINT type_check CHECK (type IN ('regular', 'voice_note', 'voice_call', 'video_call')),
  CONSTRAINT status_check CHECK (status IN ('sent', 'delivered', 'seen'))
);

-- Message Status Receipt Table
CREATE TABLE IF NOT EXISTS message_status_receipts (
     id BIGSERIAL PRIMARY KEY,
     message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
     user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
     room_id    BIGINT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
     status TEXT NOT NULL,
     created_at TIMESTAMP NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
     CONSTRAINT message_status_receipts_unique_message_user UNIQUE (message_id, user_id),
     CONSTRAINT message_status_check CHECK (status IN ('sent', 'delivered', 'seen'))
);

-- Call Logs Table
CREATE TABLE IF NOT EXISTS call_logs (
   id BIGSERIAL PRIMARY KEY,
   room_id BIGINT REFERENCES rooms(id) ON DELETE CASCADE,
   caller_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
   callee_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
   status TEXT NOT NULL DEFAULT 'missed',
   started_at VARCHAR(20) NOT NULL,
   ended_at VARCHAR(20),
   created_at TIMESTAMP NOT NULL DEFAULT NOW(),
   updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
   CONSTRAINT status_check CHECK (status IN ('missed', 'completed', 'rejected'))
);
