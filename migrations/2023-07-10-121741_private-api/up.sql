-- Your SQL goes here

CREATE TABLE customer_account (
	id bigserial primary key,
	customer_registration_id varchar NOT NULL,
	username varchar NOT NULL,
	password varchar NOT NULL,
	created_on timestamptz NOT NULL,
	password_hint varchar NOT NULL
);

CREATE TABLE customer_apikey_linking (
	id bigserial primary key,
	customer_account_id bigint NOT NULL,
	api_key varchar NOT NULL,
	api_salt_key varchar NOT NULL,
	created_on timestamptz NOT NULL,
	expires_on timestamptz NOT NULL,
	is_active boolean NOT NULL,
	remark varchar,
	authorities varchar,
	limit_remaining bigint,
	CONSTRAINT fk_user_id
		FOREIGN KEY (customer_account_id)
		REFERENCES customer_account(id)
);

CREATE TABLE customer_order_linking (
	id bigserial primary key,
	order_id uuid NOT NULL,
	public_key varchar NOT NULL,
	customer_account_id bigint NOT NULL,
	order_status varchar NOT NULL,
	created_on timestamptz NOT NULL,
	CONSTRAINT fk_customer_account_id
		FOREIGN KEY (customer_account_id)
		REFERENCES customer_account(id)
);
