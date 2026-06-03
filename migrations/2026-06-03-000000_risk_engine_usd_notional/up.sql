-- Risk-engine exposure is now netted in USD notional (im*lev*entry_price), not BTC
-- entry value. Rename the columns so the stored unit matches its name.
ALTER TABLE risk_engine_update RENAME COLUMN total_long_btc TO total_long_usd;
ALTER TABLE risk_engine_update RENAME COLUMN total_short_btc TO total_short_usd;
ALTER TABLE risk_engine_update RENAME COLUMN total_pending_long_btc TO total_pending_long_usd;
ALTER TABLE risk_engine_update RENAME COLUMN total_pending_short_btc TO total_pending_short_usd;

-- oi_series() references the old column names in its (text-stored) SQL body, so it
-- must be recreated against the renamed columns. Open interest is now USD notional.
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
           (SELECT r.total_long_usd + r.total_short_usd
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
