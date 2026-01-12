-- Add migration script here
ALTER TABLE message_status_receipts
DROP CONSTRAINT message_status_check ;

ALTER TABLE message_status_receipts
ADD CONSTRAINT message_status_check 
CHECK (status IN ('sent', 'delivered', 'seen', 'updated'));
