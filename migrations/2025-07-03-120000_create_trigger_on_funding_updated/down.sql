-- Drop the trigger
DROP TRIGGER IF EXISTS update_trader_order_trigger ON public.trader_order_funding_updated;

-- Drop the function
DROP FUNCTION IF EXISTS update_trader_order_from_funding(); 