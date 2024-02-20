-- Your SQL goes here
CREATE TABLE address_customer_id (
    id bigserial primary key,
    address varchar NOT NULL,
    customer_id bigint NOT NULL,
	CONSTRAINT fk_customer_id
		FOREIGN KEY (customer_id)
		REFERENCES customer_account(id)
)
