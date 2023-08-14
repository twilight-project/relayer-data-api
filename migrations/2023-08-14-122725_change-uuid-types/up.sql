-- Your SQL goes here
ALTER TABLE lend_pool_command ALTER COLUMN order_id TYPE varchar(64);
ALTER TABLE trader_order ALTER COLUMN uuid TYPE varchar(64);
ALTER TABLE lend_order ALTER COLUMN uuid TYPE varchar(64);
ALTER TABLE customer_order_linking ALTER COLUMN order_id TYPE varchar(64);
-- ?? This one too??
--ALTER TABLE sorted_set_command ALTER COLUMN uuid TYPE varchar(64);
