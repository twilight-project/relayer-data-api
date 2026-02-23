CREATE OR REPLACE FUNCTION get_all_trader_order_summaries(
    p_from   TIMESTAMPTZ,
    p_to     TIMESTAMPTZ,
    p_limit  BIGINT,
    p_offset BIGINT
)
RETURNS TABLE (
    twilight_address          TEXT,
    settled_positionsize      NUMERIC,
    filled_positionsize       NUMERIC,
    liquidated_positionsize   NUMERIC,
    settled_count             BIGINT,
    filled_count              BIGINT,
    liquidated_count          BIGINT
)
LANGUAGE sql
AS $$
WITH paginated_addresses AS (
    SELECT DISTINCT twilight_address
    FROM twilight_qq_account_link
    ORDER BY twilight_address
    LIMIT p_limit
    OFFSET p_offset
),
mapped_accounts AS (
    SELECT
        l.twilight_address,
        l.account_address AS q_address
    FROM twilight_qq_account_link l
    INNER JOIN paginated_addresses pa USING (twilight_address)
),
filtered_orders AS (
    SELECT
        ma.twilight_address,
        o.*
    FROM trader_order o
    INNER JOIN mapped_accounts ma ON o.account_id = ma.q_address
    WHERE o.timestamp BETWEEN p_from AND p_to
),
latest_orders AS (
    SELECT *
    FROM (
        SELECT
            fo.*,
            ROW_NUMBER() OVER (
                PARTITION BY fo.uuid
                ORDER BY fo.timestamp DESC
            ) AS rn
        FROM filtered_orders fo
    ) x
    WHERE rn = 1
)
SELECT
    lo.twilight_address,

    COALESCE(
        SUM(lo.positionsize)
        FILTER (WHERE lo.order_status = 'SETTLED'),
        0
    ) AS settled_positionsize,

    COALESCE(
        SUM(lo.positionsize)
        FILTER (WHERE lo.order_status = 'FILLED'),
        0
    ) AS filled_positionsize,

    COALESCE(
        SUM(lo.positionsize)
        FILTER (WHERE lo.order_status = 'LIQUIDATE'),
        0
    ) AS liquidated_positionsize,

    COUNT(*) FILTER (WHERE lo.order_status = 'SETTLED')   AS settled_count,
    COUNT(*) FILTER (WHERE lo.order_status = 'FILLED')    AS filled_count,
    COUNT(*) FILTER (WHERE lo.order_status = 'LIQUIDATE') AS liquidated_count
FROM latest_orders lo
GROUP BY lo.twilight_address
ORDER BY lo.twilight_address;
$$;
