CREATE INDEX IF NOT EXISTS idx_risk_engine_update_timestamp
ON risk_engine_update (timestamp DESC);

CREATE OR REPLACE FUNCTION oi_series(
  window_interval interval,
  step_interval   interval
)
RETURNS TABLE (
  bucket_ts       timestamptz,
  open_interest   float8,
  pct_change      float8
)
LANGUAGE sql STABLE
AS $$
  WITH grid AS (
    SELECT gs AS bucket_ts
    FROM generate_series(
      now() - window_interval,
      now(),
      step_interval
    ) AS gs
  ),
  snapped AS (
    SELECT g.bucket_ts,
           (SELECT r.total_long_btc + r.total_short_btc
              FROM risk_engine_update r
             WHERE r.timestamp <= g.bucket_ts
             ORDER BY r.timestamp DESC
             LIMIT 1) AS oi
    FROM grid g
  ),
  with_lag AS (
    SELECT bucket_ts,
           oi,
           LAG(oi) OVER (ORDER BY bucket_ts) AS prev_oi
    FROM snapped
  )
  SELECT bucket_ts,
         COALESCE(oi, 0.0) AS open_interest,
         CASE
           WHEN prev_oi IS NULL OR prev_oi = 0 THEN 0.0
           ELSE ((oi - prev_oi) / prev_oi) * 100.0
         END AS pct_change
  FROM with_lag
  ORDER BY bucket_ts;
$$;
