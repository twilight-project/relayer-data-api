-- This file should undo anything in `up.sql`
DROP FUNCTION IF EXISTS get_active_filled_margin_x_leverage_sum();

DROP INDEX IF EXISTS idx_trader_order_uuid_timestamp;
DROP INDEX IF EXISTS idx_trader_order_status;
DROP INDEX IF EXISTS idx_trader_order_position_type;
