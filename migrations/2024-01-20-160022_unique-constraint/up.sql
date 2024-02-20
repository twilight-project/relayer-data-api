-- Your SQL goes here
ALTER TABLE address_customer_id 
ADD CONSTRAINT id_address_unique
UNIQUE (address, customer_id);
