-- Drop fee_history table
DROP TABLE IF EXISTS public.fee_history;

-- Remove fee columns from trader_order table
ALTER TABLE public.trader_order
    DROP COLUMN IF EXISTS fee_filled,
    DROP COLUMN IF EXISTS fee_settled;

-- Remove fee columns from trader_order_funding_updated table
ALTER TABLE public.trader_order_funding_updated
    DROP COLUMN IF EXISTS fee_filled,
    DROP COLUMN IF EXISTS fee_settled; 