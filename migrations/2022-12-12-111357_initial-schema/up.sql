-- Your SQL goes here

CREATE TYPE tx_type AS ENUM ('ORDERTX', 'LENDTX');
CREATE TYPE order_type AS ENUM ('LIMIT', 'MARKET', 'DARK', 'LEND');
CREATE TYPE position_type AS ENUM ('LONG', 'SHORT');
CREATE TYPE order_status AS ENUM ('SETTLED', 'LENDED', 'LIQUIDATE', 'CANCELLED', 'PENDING', 'FILLED');

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
    uuid uuid primary key,
    account_id varchar NOT NULL,
    --position_type position_type NOT NULL,
    --order_status order_status NOT NULL,
    --order_type order_type NOT NULL,
    entryprice numeric NOT NULL,
    execution_price numeric NOT NULL,
    positionsize numeric NOT NULL,
    leverage numeric NOT NULL,
    initial_margin numeric NOT NULL,
    available_margin numeric NOT NULL,
    timestamp timestamptz,
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
    uuid uuid primary key,
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
