-- Fix co_member type mismatch (INT4 → INT8)

ALTER TABLE rooms
    DROP CONSTRAINT IF EXISTS fk_co_member;

ALTER TABLE rooms
    ALTER COLUMN co_member TYPE BIGINT;

ALTER TABLE rooms
    ADD CONSTRAINT fk_co_member
        FOREIGN KEY (co_member)
            REFERENCES users(id);
