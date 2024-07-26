-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS public.trader_order_funding_updated;

-- Index: timestamp_trader_order_funding_updated

DROP INDEX IF EXISTS public.timestamp_trader_order_funding_updated;

-- Index: trader_order_funding_updated_uuid

DROP INDEX IF EXISTS public.trader_order_funding_updated_uuid;