-- Drop the trigger
DROP TRIGGER IF EXISTS update_trader_order_trigger ON public.trader_order_funding_updated;

-- Drop the function
DROP FUNCTION IF EXISTS update_trader_order_from_funding(); 


DROP VIEW IF EXISTS orderbook;


DROP INDEX IF EXISTS sorted_set_command_cmd_uuid;
DROP INDEX IF EXISTS trader_order_open_limit_idx;
DROP INDEX IF EXISTS trader_order_uuid_id_desc;
DROP INDEX IF EXISTS sorted_set_command_uuid_id_desc_close_limit;
