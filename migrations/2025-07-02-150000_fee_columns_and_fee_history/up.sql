-- Add fee columns to trader_order table
ALTER TABLE public.trader_order
    ADD COLUMN IF NOT EXISTS fee_filled numeric NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS fee_settled numeric NOT NULL DEFAULT 0;

-- Add fee columns to trader_order_funding_updated table
ALTER TABLE public.trader_order_funding_updated
    ADD COLUMN IF NOT EXISTS fee_filled numeric NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS fee_settled numeric NOT NULL DEFAULT 0;

-- Create fee_history table
CREATE TABLE IF NOT EXISTS public.fee_history
(
    id bigint NOT NULL GENERATED ALWAYS AS IDENTITY,
    order_filled_on_market numeric NOT NULL,
    order_filled_on_limit numeric NOT NULL,
    order_settled_on_market numeric NOT NULL,
    order_settled_on_limit numeric NOT NULL,
    "timestamp" timestamptz NOT NULL,
    CONSTRAINT fee_history_pkey PRIMARY KEY (id)
);

-- Index to speed up time based queries
CREATE INDEX IF NOT EXISTS fee_history_timestamp_idx
    ON public.fee_history USING btree
    ("timestamp" ASC NULLS LAST); 