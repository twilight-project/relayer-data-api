-- Your SQL goes here
-- ============================================================
-- Indexes (critical for window function performance)
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_trader_order_uuid_timestamp
ON trader_order (uuid, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_trader_order_status
ON trader_order (order_status);

CREATE INDEX IF NOT EXISTS idx_trader_order_position_type
ON trader_order (position_type);

-- ============================================================
-- Function: Global active FILLED margin × leverage exposure
-- Returns:
--   - long_exposure
--   - short_exposure
--   - last_order_timestamp (latest across all active FILLED orders)
-- ============================================================

CREATE OR REPLACE FUNCTION get_active_filled_margin_x_leverage_sum()
RETURNS TABLE (
    long_exposure        NUMERIC,
    short_exposure       NUMERIC,
    last_order_timestamp TIMESTAMPTZ
)
LANGUAGE sql
AS $$
WITH latest_orders AS (
    SELECT *
    FROM (
        SELECT
            t.*,
            ROW_NUMBER() OVER (
                PARTITION BY t.uuid
                ORDER BY t.timestamp DESC
            ) AS rn
        FROM trader_order t
    ) x
    WHERE rn = 1
),
active_filled_orders AS (
    SELECT *
    FROM latest_orders
    WHERE order_status = 'FILLED'
)
SELECT
    COALESCE(
        SUM(initial_margin * leverage)
            FILTER (WHERE position_type = 'LONG'),
        0
    ) AS long_exposure,

    COALESCE(
        SUM(initial_margin * leverage)
            FILTER (WHERE position_type = 'SHORT'),
        0
    ) AS short_exposure,

    MAX(timestamp) AS last_order_timestamp
FROM active_filled_orders;
$$;
