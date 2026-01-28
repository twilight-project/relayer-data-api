-- This file should undo anything in `up.sql`
DROP FUNCTION IF EXISTS get_trader_order_summary_by_t_address(
    TEXT,
    TIMESTAMPTZ,
    TIMESTAMPTZ
);

DROP INDEX IF EXISTS idx_trader_order_account_timestamp;
DROP INDEX IF EXISTS idx_trader_order_uuid_timestamp;
DROP INDEX IF EXISTS idx_trader_order_status;
DROP INDEX IF EXISTS idx_addr_mappings_t_address;

