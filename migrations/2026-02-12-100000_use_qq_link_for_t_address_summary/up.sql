-- Replace get_trader_order_summary_by_t_address to use
-- twilight_qq_account_link instead of chain_indexer.addr_mappings.
--
-- Column mapping:
--   chain_indexer.addr_mappings.t_address  -> twilight_qq_account_link.twilight_address
--   chain_indexer.addr_mappings.q_address  -> twilight_qq_account_link.account_address

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
    SELECT DISTINCT account_address AS q_address
    FROM twilight_qq_account_link
    WHERE twilight_address = p_t_address
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

-- Index for the new lookup path
CREATE INDEX IF NOT EXISTS idx_qq_link_twilight_address
ON twilight_qq_account_link (twilight_address);

-- Drop the old index on chain_indexer.addr_mappings (no longer needed)
DROP INDEX IF EXISTS idx_addr_mappings_t_address;
