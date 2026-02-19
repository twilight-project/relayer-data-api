-- Drop the function
DROP FUNCTION IF EXISTS get_trader_order_summary_by_t_address(
    TEXT,
    TIMESTAMPTZ,
    TIMESTAMPTZ
);

-- Drop all indexes created in up.sql
DROP INDEX IF EXISTS idx_trader_order_account_timestamp;
DROP INDEX IF EXISTS idx_trader_order_uuid_timestamp;
DROP INDEX IF EXISTS idx_trader_order_status;
DROP INDEX IF EXISTS idx_qq_link_twilight_address;
