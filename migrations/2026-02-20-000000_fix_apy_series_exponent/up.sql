-- Fix apy_series(): replace hardcoded exponent 365 with a generalised
-- EXTRACT(EPOCH)-based formula so that non-24h lookbacks (7d, 30d) produce
-- the correct annualised return.
--
-- Formula: POWER(ratio, 365_days_in_seconds / lookback_in_seconds) - 1
--
-- Examples:
--   lookback = '24 hours'  → exponent = 31536000 / 86400   = 365
--   lookback = '7 days'    → exponent = 31536000 / 604800  ≈ 52.14
--   lookback = '30 days'   → exponent = 31536000 / 2592000 ≈ 12.17
--
-- last_day_apy_now() uses a fixed '24 hours' lookback, so its exponent is
-- always 365 — no change needed there.

CREATE OR REPLACE FUNCTION apy_series(
  window_interval   interval,           -- e.g. '24 hours' | '7 days' | '30 days'
  step_interval     interval,           -- e.g. '1 minute' | '5 minutes' | '1 hour'
  lookback_interval interval DEFAULT '24 hours'
)
RETURNS TABLE (
  bucket_ts timestamptz,
  apy       NUMERIC
)
LANGUAGE sql
STABLE
AS $$
  WITH bounds AS (
    SELECT now() AT TIME ZONE 'utc' AS t_end
  ),
  grid AS (
    SELECT gs AS bucket_ts
    FROM bounds, LATERAL generate_series(
      (SELECT t_end FROM bounds) - window_interval,
      (SELECT t_end FROM bounds),
      step_interval
    ) AS gs
  ),
  p_now AS (
    SELECT g.bucket_ts,
           (SELECT share_price
              FROM lend_pool_price_minute p
             WHERE p.bucket_ts <= g.bucket_ts
             ORDER BY p.bucket_ts DESC
             LIMIT 1) AS sp_now
    FROM grid g
  ),
  p_then AS (
    SELECT g.bucket_ts,
           (SELECT share_price
              FROM lend_pool_price_minute p
             WHERE p.bucket_ts <= g.bucket_ts - lookback_interval
             ORDER BY p.bucket_ts DESC
             LIMIT 1) AS sp_then
    FROM grid g
  )
  SELECT g.bucket_ts,
         CASE
           WHEN n.sp_now IS NULL OR t.sp_then IS NULL OR t.sp_then = 0
             THEN NULL
           ELSE POWER(
             n.sp_now / t.sp_then,
             EXTRACT(EPOCH FROM interval '365 days') / EXTRACT(EPOCH FROM lookback_interval)
           ) - 1
         END AS apy
  FROM grid g
  JOIN p_now  n USING (bucket_ts)
  JOIN p_then t USING (bucket_ts)
  ORDER BY g.bucket_ts;
$$;
