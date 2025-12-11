-- Add migration script here
DROP TABLE IF EXISTS call_logs;

CREATE TABLE call_logs (
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
