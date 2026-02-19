-- Repair candle data: truncate and repopulate from raw btc_usd_price + trader_order.
-- Uses the fixed get_candles_interval function (correct OHLC, correct volume, correct bucketing).
--
-- Prerequisites:
--   1. Run the migration 2026-02-10-100000_fix_candle_aggregation first
--      (diesel migration revert && diesel migration run)
--   2. Then run this script:  psql -d <database> -f scripts/repair_candles.sql
--
-- This may take several minutes depending on data volume.

BEGIN;

-- 1. Empty all candle tables
TRUNCATE candles_1min;
TRUNCATE candles_1hour;
TRUNCATE candles_1day;

-- 2. Repopulate 1-day candles (smallest result set, fastest)
INSERT INTO candles_1day (start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close)
SELECT * FROM get_candles_interval(
    '1 day'::interval,
    '1970-01-01 00:00:00+00'::timestamptz
);

-- 3. Repopulate 1-hour candles
INSERT INTO candles_1hour (start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close)
SELECT * FROM get_candles_interval(
    '1 hour'::interval,
    '1970-01-01 00:00:00+00'::timestamptz
);

-- 4. Repopulate 1-minute candles (largest result set, slowest)
INSERT INTO candles_1min (start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close)
SELECT * FROM get_candles_interval(
    '1 minute'::interval,
    '1970-01-01 00:00:00+00'::timestamptz
);

COMMIT;
