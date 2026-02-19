-- 1) Price history table (one row per minute)
CREATE TABLE IF NOT EXISTS lend_pool_price_minute (
  bucket_ts           timestamptz PRIMARY KEY,          -- minute bucket (UTC)
  share_price         NUMERIC(38,18) NOT NULL,          -- TLV / TPS
  total_locked_value  NUMERIC(38,18) NOT NULL,
  total_pool_share    NUMERIC(38,18) NOT NULL,
  samples             integer NOT NULL DEFAULT 1,
  source              text NOT NULL DEFAULT 'trigger'
);

-- PK already creates a btree index; no extra index is necessary.


-- 2) Trigger function: upsert one snapshot per minute
CREATE OR REPLACE FUNCTION upsert_lend_pool_price_minute()
RETURNS trigger
LANGUAGE plpgsql
AS $fn$
DECLARE
  b_ts timestamptz;
  p    NUMERIC(38,18);
BEGIN
  -- Skip when price is undefined
  IF NEW.total_pool_share = 0 THEN
    RETURN NULL;
  END IF;

  b_ts := date_trunc('minute', (now() AT TIME ZONE 'utc'));
  p    := NEW.total_locked_value / NEW.total_pool_share;

  INSERT INTO lend_pool_price_minute (
    bucket_ts, share_price, total_locked_value, total_pool_share, samples, source
  )
  VALUES (b_ts, p, NEW.total_locked_value, NEW.total_pool_share, 1, 'trigger')
  ON CONFLICT (bucket_ts) DO UPDATE
  SET
    share_price         = EXCLUDED.share_price,
    total_locked_value  = EXCLUDED.total_locked_value,
    total_pool_share    = EXCLUDED.total_pool_share,
    samples             = lend_pool_price_minute.samples + 1,
    source              = 'trigger';

  RETURN NULL;
END
$fn$;


-- 3) Attach triggers to `lend_pool`
-- Clean old triggers if they exist (safe in up.sql too)
DROP TRIGGER IF EXISTS lend_pool_price_minute_ins_trg ON lend_pool;
DROP TRIGGER IF EXISTS lend_pool_price_minute_upd_trg ON lend_pool;

-- Fire after each INSERT (append-only pipelines)
CREATE TRIGGER lend_pool_price_minute_ins_trg
AFTER INSERT ON lend_pool
FOR EACH ROW
EXECUTE FUNCTION upsert_lend_pool_price_minute();

-- Fire after UPDATE of price-relevant columns (if you edit the latest row)
CREATE TRIGGER lend_pool_price_minute_upd_trg
AFTER UPDATE OF total_locked_value, total_pool_share ON lend_pool
FOR EACH ROW
WHEN (NEW.total_locked_value IS DISTINCT FROM OLD.total_locked_value
   OR NEW.total_pool_share   IS DISTINCT FROM OLD.total_pool_share)
EXECUTE FUNCTION upsert_lend_pool_price_minute();


-- 4) Reusable function to build an APY time series (for charts)
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
           ELSE POWER(n.sp_now / t.sp_then, 365) - 1
         END AS apy
  FROM grid g
  JOIN p_now  n USING (bucket_ts)
  JOIN p_then t USING (bucket_ts)
  ORDER BY g.bucket_ts;
$$;


-- 5) Optional convenience: last-day APY at "now()"
CREATE OR REPLACE FUNCTION last_day_apy_now()
RETURNS NUMERIC
LANGUAGE sql
STABLE
AS $$
  SELECT CASE
    WHEN p_now.sp IS NULL OR p_then.sp IS NULL OR p_then.sp = 0 THEN NULL
    ELSE POWER(p_now.sp / p_then.sp, 365) - 1
  END
  FROM
    LATERAL (
      SELECT share_price AS sp
      FROM lend_pool_price_minute
      WHERE bucket_ts <= (now() AT TIME ZONE 'utc')
      ORDER BY bucket_ts DESC
      LIMIT 1
    ) AS p_now,
    LATERAL (
      SELECT share_price AS sp
      FROM lend_pool_price_minute
      WHERE bucket_ts <= (now() AT TIME ZONE 'utc' - interval '24 hours')
      ORDER BY bucket_ts DESC
      LIMIT 1
    ) AS p_then;
$$;
