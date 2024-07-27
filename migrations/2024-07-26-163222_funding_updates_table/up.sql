-- Your SQL goes here

-- DROP TABLE IF EXISTS public.trader_order_funding_updated;

CREATE TABLE IF NOT EXISTS public.trader_order_funding_updated
(
    id bigint NOT NULL DEFAULT nextval('trader_order_id_seq'::regclass),
    uuid character varying(64) COLLATE pg_catalog."default" NOT NULL,
    account_id character varying COLLATE pg_catalog."default" NOT NULL,
    position_type position_type NOT NULL,
    order_status order_status NOT NULL,
    order_type order_type NOT NULL,
    entryprice numeric NOT NULL,
    execution_price numeric NOT NULL,
    positionsize numeric NOT NULL,
    leverage numeric NOT NULL,
    initial_margin numeric NOT NULL,
    available_margin numeric NOT NULL,
    "timestamp" timestamp with time zone NOT NULL,
    bankruptcy_price numeric NOT NULL,
    bankruptcy_value numeric NOT NULL,
    maintenance_margin numeric NOT NULL,
    liquidation_price numeric NOT NULL,
    unrealized_pnl numeric NOT NULL,
    settlement_price numeric NOT NULL,
    entry_nonce bigint NOT NULL,
    exit_nonce bigint NOT NULL,
    entry_sequence bigint NOT NULL,
    CONSTRAINT trader_order_funding_updated_pkey PRIMARY KEY (id)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.trader_order_funding_updated
    OWNER to relayer;
-- Index: timestamp_trader_order_funding_updated

-- DROP INDEX IF EXISTS public.timestamp_trader_order_funding_updated;

CREATE INDEX IF NOT EXISTS timestamp_trader_order_funding_updated
    ON public.trader_order_funding_updated USING btree
    ("timestamp" ASC NULLS LAST)
    TABLESPACE pg_default;
-- Index: trader_order_funding_updated_uuid

-- DROP INDEX IF EXISTS public.trader_order_funding_updated_uuid;

CREATE INDEX IF NOT EXISTS trader_order_funding_updated_uuid
    ON public.trader_order_funding_updated USING btree
    (uuid COLLATE pg_catalog."default" ASC NULLS LAST)
    TABLESPACE pg_default;