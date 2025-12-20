-- Add migration script here
ALTER TABLE rooms
    ADD COLUMN co_member INT UNIQUE, -- for private-chats only
    ADD CONSTRAINT fk_co_member FOREIGN KEY (co_member) REFERENCES users(id);