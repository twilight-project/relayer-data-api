-- Your SQL goes here
CREATE TABLE transaction_hash (
    id bigserial primary key,
    order_id varchar NOT NULL,
    account_id varchar NOT NULL,
    tx_hash varchar NOT NULL,
    order_type order_type NOT NULL,
    order_status order_status NOT NULL,
    datetime varchar NOT NULL,
    output varchar
);

ALTER TABLE sorted_set_command ALTER COLUMN uuid TYPE varchar;
