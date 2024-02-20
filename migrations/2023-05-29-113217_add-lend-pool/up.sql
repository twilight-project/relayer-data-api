-- Your SQL goes here

CREATE TABLE lend_pool (
	id bigserial primary key,
	sequence bigint NOT NULL,
	nonce bigint NOT NULL,
	total_pool_share numeric NOT NULL,
	total_locked_value numeric NOT NULL,
	pending_orders bigint NOT NULL,
	aggregate_log_sequence bigint NOT NULL,
	last_snapshot_id bigint NOT NULL
);
