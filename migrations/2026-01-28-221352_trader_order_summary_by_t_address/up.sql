-- Your SQL goes here
-- ============================================================
-- Function:
--   get_trader_order_summary_by_t_address
--
-- Description:
--   For a given t_address and date range:
--   - Resolve all q_address mapped to t_address
--   - Consider ONLY trader_order.timestamp for date filtering
--   - Take latest row per uuid
--   - Aggregate positionsize + counts by final order_status
-- ============================================================

CREATE OR REPLACE FUNCTION get_trader_order_summary_by_t_address(
    p_t_address TEXT,
    p_from TIMESTAMPTZ,
    p_to   TIMESTAMPTZ
)
RETURNS TABLE (
    settled_positionsize      NUMERIC,
    filled_positionsize       NUMERIC,
    liquidated_positionsize   NUMERIC,
    settled_count             BIGINT,
    filled_count              BIGINT,
    liquidated_count          BIGINT
)
LANGUAGE sql
AS $$
WITH mapped_accounts AS (
    SELECT q_address
    FROM chain_indexer.addr_mappings
    WHERE t_address = p_t_address
),
filtered_orders AS (
    SELECT *
    FROM trader_order
    WHERE
        account_id IN (SELECT q_address FROM mapped_accounts)
        AND timestamp BETWEEN p_from AND p_to
),
latest_orders AS (
    SELECT *
    FROM (
        SELECT
            o.*,
            ROW_NUMBER() OVER (
                PARTITION BY o.uuid
                ORDER BY o.timestamp DESC
            ) AS rn
        FROM filtered_orders o
    ) x
    WHERE rn = 1
)
SELECT
    -- SETTLED
    COALESCE(
        SUM(positionsize)
        FILTER (WHERE order_status = 'SETTLED'),
        0
    ) AS settled_positionsize,

    -- FILLED but not settled later (latest state = FILLED)
    COALESCE(
        SUM(positionsize)
        FILTER (WHERE order_status = 'FILLED'),
        0
    ) AS filled_positionsize,

    -- LIQUIDATED
    COALESCE(
        SUM(positionsize)
        FILTER (WHERE order_status = 'LIQUIDATE'),
        0
    ) AS liquidated_positionsize,

    -- COUNTS
    COUNT(*) FILTER (WHERE order_status = 'SETTLED')   AS settled_count,
    COUNT(*) FILTER (WHERE order_status = 'FILLED')    AS filled_count,
    COUNT(*) FILTER (WHERE order_status = 'LIQUIDATE') AS liquidated_count
FROM latest_orders;
$$;

-- ============================================================
-- Indexes (performance)
-- ============================================================

CREATE INDEX IF NOT EXISTS idx_trader_order_account_timestamp
ON trader_order (account_id, timestamp);

CREATE INDEX IF NOT EXISTS idx_trader_order_uuid_timestamp
ON trader_order (uuid, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_trader_order_status
ON trader_order (order_status);

CREATE INDEX IF NOT EXISTS idx_addr_mappings_t_address
ON chain_indexer.addr_mappings (t_address);
