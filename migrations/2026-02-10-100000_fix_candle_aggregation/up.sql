-- Fix candle aggregation bugs:
--
-- 1. Open/Close SWAPPED in get_ohlc_interval: first_value(price) was labeled 'close',
--    last_value(price) was labeled 'open'. With ORDER BY timestamp ASC, first_value is
--    the earliest price (= open) and last_value is the latest (= close).
--
-- 2. SELECT DISTINCT + window functions instead of GROUP BY: fragile, non-deterministic
--    pattern. Replaced with proper GROUP BY + aggregate functions.
--
-- 3. date_trunc() cannot produce arbitrary-interval buckets (e.g. 4 hours).
--    Replaced with epoch-based floor division that works for any interval.
--
-- 4. Column alias typo in get_candles_interval: end_time was aliased as 'start_time'.
--
-- 5. get_volume_interval had same DISTINCT + window anti-pattern.

-- Drop old functions (order matters: dependents first)
DROP FUNCTION IF EXISTS update_candles_1day();
DROP FUNCTION IF EXISTS update_candles_1hour();
DROP FUNCTION IF EXISTS update_candles_1min();
DROP FUNCTION IF EXISTS get_candles_interval(interval, text, timestamptz);
DROP FUNCTION IF EXISTS get_volume_interval(interval, text, timestamptz);
DROP FUNCTION IF EXISTS get_ohlc_interval(interval, text, timestamptz);


-- OHLC price aggregation: GROUP BY with array_agg for first/last price
CREATE FUNCTION get_ohlc_interval(intvl interval, since timestamptz)
RETURNS TABLE(start_time timestamptz, end_time timestamptz, high numeric, low numeric, open numeric, close numeric)
AS $$
    SELECT
        bucket                  as start_time,
        bucket + intvl          as end_time,
        max(price)              as high,
        min(price)              as low,
        (array_agg(price ORDER BY timestamp ASC ))[1] as open,
        (array_agg(price ORDER BY timestamp DESC))[1] as close
    FROM (
        SELECT
            to_timestamp(
                floor(extract(epoch from timestamp) / extract(epoch from intvl))
                * extract(epoch from intvl)
            ) as bucket,
            price,
            timestamp
        FROM btc_usd_price
        WHERE timestamp >= since AND timestamp <= now()
    ) t
    GROUP BY bucket
    ORDER BY bucket
$$
LANGUAGE SQL;


-- Volume aggregation: GROUP BY instead of DISTINCT + window
CREATE FUNCTION get_volume_interval(intvl interval, since timestamptz)
RETURNS TABLE(start_time timestamptz, end_time timestamptz, usd_volume numeric, btc_volume numeric, trades integer)
AS $$
    SELECT
        bucket                              as start_time,
        bucket + intvl                      as end_time,
        coalesce(sum(entryprice), 0)        as usd_volume,
        coalesce(sum(positionsize), 0)      as btc_volume,
        count(*)::integer                   as trades
    FROM (
        SELECT
            to_timestamp(
                floor(extract(epoch from timestamp) / extract(epoch from intvl))
                * extract(epoch from intvl)
            ) as bucket,
            entryprice,
            positionsize
        FROM trader_order
        WHERE timestamp >= since AND timestamp <= now()
    ) t
    GROUP BY bucket
    ORDER BY bucket
$$
LANGUAGE SQL;


-- Combined candle function: FULL OUTER JOIN of OHLC + Volume
-- Fixed: end_time alias was incorrectly 'start_time'
CREATE FUNCTION get_candles_interval(intvl interval, since timestamptz)
RETURNS TABLE(
    start_time timestamptz,
    end_time   timestamptz,
    usd_volume numeric,
    btc_volume numeric,
    trades     integer,
    open       numeric,
    high       numeric,
    low        numeric,
    close      numeric
)
AS $$
WITH t1 AS (
    SELECT * FROM get_ohlc_interval(intvl, since)
), t2 AS (
    SELECT * FROM get_volume_interval(intvl, since)
) SELECT
    coalesce(t1.start_time, t2.start_time) as start_time,
    coalesce(t1.end_time, t2.end_time)     as end_time,
    coalesce(t2.usd_volume, 0)             as usd_volume,
    coalesce(t2.btc_volume, 0)             as btc_volume,
    coalesce(t2.trades, 0)                 as trades,
    coalesce(t1.open, 0)                   as open,
    coalesce(t1.high, 0)                   as high,
    coalesce(t1.low, 0)                    as low,
    coalesce(t1.close, 0)                  as close
FROM
    t1 FULL OUTER JOIN t2
    ON t1.start_time = t2.start_time
$$
LANGUAGE SQL;


-- Materialization functions (updated to 2-param signatures)

CREATE FUNCTION update_candles_1min()
RETURNS void
AS $$ INSERT INTO candles_1min (
    start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close
)
SELECT * FROM get_candles_interval(
    '1 minute'::interval,
    (SELECT coalesce(max(start_time) - interval '10 minute',
                     '1970-01-01 00:00:00.0000+00'::timestamptz) FROM candles_1min)
)
    ON CONFLICT(start_time)
    DO UPDATE SET
    start_time = excluded.start_time,
    end_time   = excluded.end_time,
    usd_volume = excluded.usd_volume,
    btc_volume = excluded.btc_volume,
    trades     = excluded.trades,
    open       = excluded.open,
    high       = excluded.high,
    low        = excluded.low,
    close      = excluded.close
    ;
$$
LANGUAGE SQL;

CREATE FUNCTION update_candles_1hour()
RETURNS void
AS $$ INSERT INTO candles_1hour (
    start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close
)
SELECT * FROM get_candles_interval(
    '1 hour'::interval,
    (SELECT coalesce(max(start_time) - interval '10 minute',
                     '1970-01-01 00:00:00.0000+00'::timestamptz) FROM candles_1hour)
)
    ON CONFLICT(start_time)
    DO UPDATE SET
    start_time = excluded.start_time,
    end_time   = excluded.end_time,
    usd_volume = excluded.usd_volume,
    btc_volume = excluded.btc_volume,
    trades     = excluded.trades,
    open       = excluded.open,
    high       = excluded.high,
    low        = excluded.low,
    close      = excluded.close
    ;
$$
LANGUAGE SQL;

CREATE FUNCTION update_candles_1day()
RETURNS void
AS $$ INSERT INTO candles_1day (
    start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close
)
SELECT * FROM get_candles_interval(
    '1 day'::interval,
    (SELECT coalesce(max(start_time) - interval '10 minute',
                     '1970-01-01 00:00:00.0000+00'::timestamptz) FROM candles_1day)
)
    ON CONFLICT(start_time)
    DO UPDATE SET
    start_time = excluded.start_time,
    end_time   = excluded.end_time,
    usd_volume = excluded.usd_volume,
    btc_volume = excluded.btc_volume,
    trades     = excluded.trades,
    open       = excluded.open,
    high       = excluded.high,
    low        = excluded.low,
    close      = excluded.close
    ;
$$
LANGUAGE SQL;
