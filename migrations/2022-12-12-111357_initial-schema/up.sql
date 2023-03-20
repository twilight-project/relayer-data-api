-- Your SQL goes here

CREATE TYPE tx_type AS ENUM ('ORDERTX', 'LENDTX');
CREATE TYPE order_type AS ENUM ('LIMIT', 'MARKET', 'DARK', 'LEND');
CREATE TYPE position_type AS ENUM ('LONG', 'SHORT');
CREATE TYPE order_status AS ENUM ('SETTLED', 'LENDED', 'LIQUIDATE', 'CANCELLED', 'PENDING', 'FILLED');
CREATE TYPE position_size_command AS ENUM ('ADD', 'REMOVE');
CREATE TYPE sorted_set_command_type AS ENUM (
	'ADD_LIQUIDATION_PRICE',
	'ADD_OPEN_LIMIT_PRICE',
	'ADD_CLOSE_LIMIT_PRICE',
        'REMOVE_LIQUIDATION_PRICE',
        'REMOVE_OPEN_LIMIT_PRICE',
        'REMOVE_CLOSE_LIMIT_PRICE',
        'UPDATE_LIQUIDATION_PRICE',
        'UPDATE_OPEN_LIMIT_PRICE',
        'UPDATE_CLOSE_LIMIT_PRICE',
        'BULK_SEARCH_REMOVE_LIQUIDATION_PRICE',
        'BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE',
        'BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE'
);
CREATE TYPE lend_pool_command_type AS ENUM (
	'ADD_TRADER_ORDER_SETTLEMENT',
	'ADD_TRADER_LIMIT_ORDER_SETTLEMENT',
	'ADD_FUNDING_DATA',
	'ADD_TRADER_ORDER_LIQUIDATION',
	'LEND_ORDER_CREATE_ORDER',
	'LEND_ORDER_SETTLE_ORDER',
	'BATCH_EXECUTE_TRADER_ORDER',
	'INITIATE_NEW_POOL'
);

CREATE TABLE lend_pool_command (
	id bigserial primary key,
	command lend_pool_command_type NOT NULL,
	order_id uuid NOT NULL,
	payment numeric
);

CREATE TABLE sorted_set_command (
	id bigserial primary key,
	command sorted_set_command_type NOT NULL,
	uuid uuid,
	amount numeric,
	position_type position_type NOT NULL
);

CREATE TABLE position_size_log (
	id bigserial primary key,
	command position_size_command NOT NULL,
	position_type position_type NOT NULL,
	amount numeric NOT NULL,
	total_short numeric NOT NULL,
	total_long numeric NOT NULL,
	total numeric NOT NULL
);

CREATE TABLE btc_usd_price (
    id bigserial primary key,
    price numeric NOT NULL,
    timestamp timestamptz NOT NULL
);

CREATE TABLE funding_rate (
	id bigserial primary key,
	rate numeric NOT NULL,
	price numeric NOT NULL,
	timestamp timestamptz NOT NULL
);

CREATE TABLE trader_order (
    id bigserial primary key,
    uuid uuid NOT NULL,
    account_id varchar NOT NULL,
    position_type position_type NOT NULL,
    order_status order_status NOT NULL,
    order_type order_type NOT NULL,
    entryprice numeric NOT NULL,
    execution_price numeric NOT NULL,
    positionsize numeric NOT NULL,
    leverage numeric NOT NULL,
    initial_margin numeric NOT NULL,
    available_margin numeric NOT NULL,
    timestamp timestamptz NOT NULL,
    bankruptcy_price numeric NOT NULL,
    bankruptcy_value numeric NOT NULL,
    maintenance_margin numeric NOT NULL,
    liquidation_price numeric NOT NULL,
    unrealized_pnl numeric NOT NULL,
    settlement_price numeric NOT NULL,
    entry_nonce bigint NOT NULL,
    exit_nonce bigint NOT NULL,
    entry_sequence bigint NOT NULL
);


CREATE TABLE lend_order (
    id bigserial primary key,
    uuid uuid NOT NULL,
    account_id varchar NOT NULL,
    balance numeric NOT NULL,
    order_status order_status NOT NULL,
    order_type order_type NOT NULL,
    entry_nonce bigint NOT NULL,
    exit_nonce bigint NOT NULL,
    deposit numeric NOT NULL,
    new_lend_state_amount numeric NOT NULL,
    timestamp timestamptz NOT NULL,
    npoolshare numeric NOT NULL,
    nwithdraw numeric NOT NULL,
    payment numeric NOT NULL,
    tlv0 numeric NOT NULL,
    tps0 numeric NOT NULL,
    tlv1 numeric NOT NULL,
    tps1 numeric NOT NULL,
    tlv2 numeric NOT NULL,
    tps2 numeric NOT NULL,
    tlv3 numeric NOT NULL,
    tps3 numeric NOT NULL,
    entry_sequence bigint NOT NULL
);
