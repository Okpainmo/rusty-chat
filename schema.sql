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
    pinned_by BIGINT[] DEFAULT '{}',
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
CREATE INDEX IF NOT EXISTS idx_rooms_pinned_by ON rooms USING GIN(pinned_by);

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
  text_content TEXT,
  attachment_1 TEXT,
  attachment_2 TEXT,
  attachment_3 TEXT,
  attachment_4 TEXT,
  status TEXT NOT NULL DEFAULT 'sent',
  sent_at VARCHAR(20) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updates_counter INTEGER NOT NULL DEFAULT 0,
  CONSTRAINT updates_counter_check CHECK (updates_counter >= 0),
  CONSTRAINT type_check CHECK (type IN ('regular', 'voice_note', 'voice_call', 'video_call')),
  CONSTRAINT status_check CHECK (status IN ('sent', 'delivered', 'seen', 'updated'))
);

-- Message Status Receipt Table
CREATE TABLE IF NOT EXISTS message_status_receipts (
     id BIGSERIAL PRIMARY KEY,
     message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
     room_id    BIGINT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
     sender_id  BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
     receiver_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
     status TEXT NOT NULL,
     action TEXT NOT NULL,
     created_at TIMESTAMP NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
     updates_count_tracker INTEGER NOT NULL DEFAULT 0,
     CONSTRAINT updates_count_tracker_check CHECK (updates_count_tracker >= 0),
     CONSTRAINT message_status_check CHECK (status IN ('sent', 'delivered', 'seen', 'updated')),
     CONSTRAINT message_status_receipts_action_check CHECK (action IN ('original-send', 'edit', 'delete', 'reaction', 'system'))
);

-- Index for message_status_receipts.action
CREATE INDEX IF NOT EXISTS idx_message_status_receipts_action ON message_status_receipts (action);

-- Index for message_status_receipts history lookups
CREATE INDEX IF NOT EXISTS idx_message_status_receipts_message ON message_status_receipts (message_id, created_at);
CREATE INDEX IF NOT EXISTS idx_message_receipts_update_tracking ON message_status_receipts (message_id, updates_count_tracker);

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

-- Message Bookmarks Table
CREATE TABLE IF NOT EXISTS message_bookmarks (
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, message_id)
);

CREATE INDEX IF NOT EXISTS idx_message_bookmarks_user ON message_bookmarks (user_id);
CREATE INDEX IF NOT EXISTS idx_message_bookmarks_message ON message_bookmarks (message_id);

-- Message Archives Table
CREATE TABLE IF NOT EXISTS message_archives (
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, message_id)
);

CREATE INDEX IF NOT EXISTS idx_message_archives_user ON message_archives (user_id);
CREATE INDEX IF NOT EXISTS idx_message_archives_message ON message_archives (message_id);

-- Message Edits Table
CREATE TABLE IF NOT EXISTS message_edits (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    previous_context TEXT NOT NULL,
    new_content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_edits_message_id ON message_edits (message_id);
CREATE INDEX IF NOT EXISTS idx_message_edits_created_at ON message_edits (created_at);
