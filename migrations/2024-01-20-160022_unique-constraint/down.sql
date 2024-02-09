-- This file should undo anything in `up.sql`
ALTER TABLE address_customer_id 
DROP CONSTRAINT id_address_unique;
