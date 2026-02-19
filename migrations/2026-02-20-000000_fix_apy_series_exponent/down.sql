-- Revert apy_series() to the original hardcoded-365 formula.
CREATE OR REPLACE FUNCTION apy_series(
  window_interval   interval,
  step_interval     interval,
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
           ELSE POWER(n.sp_now / t.sp_then, 365) - 1
         END AS apy
  FROM grid g
  JOIN p_now  n USING (bucket_ts)
  JOIN p_then t USING (bucket_ts)
  ORDER BY g.bucket_ts;
$$;
