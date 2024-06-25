-- Your SQL goes here

-- CREATE INDEX trader_order_time ON trader_order(timestamp);
-- CREATE INDEX price_time ON btc_usd_price(timestamp);

CREATE INDEX trader_order_uuid ON trader_order(uuid);
CREATE INDEX transaction_hash_account_id ON transaction_hash(account_id);
CREATE INDEX transaction_hash_request_id ON transaction_hash(request_id);

CREATE TABLE candles_1min (
    start_time timestamptz primary key,
    end_time timestamptz not null,
    low numeric not null,
    high numeric not null,
    open numeric not null,
    close numeric not null,
    trades integer not null,
    btc_volume numeric not null,
    usd_volume numeric not null
);

CREATE TABLE candles_1hour (
    start_time timestamptz primary key,
    end_time timestamptz not null,
    low numeric not null,
    high numeric not null,
    open numeric not null,
    close numeric not null,
    trades integer not null,
    btc_volume numeric not null,
    usd_volume numeric not null
);

CREATE TABLE candles_1day (
    start_time timestamptz primary key,
    end_time timestamptz not null,
    low numeric not null,
    high numeric not null,
    open numeric not null,
    close numeric not null,
    trades integer not null,
    btc_volume numeric not null,
    usd_volume numeric not null
);

CREATE FUNCTION get_ohlc_interval(intvl interval, trunc_by text, since timestamptz)
RETURNS TABLE(start_time timestamptz, end_time timestamptz, high numeric, low numeric, open numeric, close numeric)
AS $$ SELECT DISTINCT
        bucket as start_time,
        bucket + intvl - interval '1 microsecond' as end_time,
        max(price) over w as high,
        min(price) over w as low,
        first_value(price) over w as close,
        last_value(price) over w as open
from (
    SELECT date_trunc(trunc_by, timestamp) as bucket,*
    FROM btc_usd_price
    WHERE timestamp BETWEEN since AND now() 
) as t
WINDOW w as (partition by bucket order by timestamp asc ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
$$
LANGUAGE SQL;


CREATE FUNCTION get_volume_interval(intvl interval, trunc_by text, since timestamptz)
RETURNS TABLE(start_time timestamptz, end_time timestamptz, usd_volume numeric, btc_volume numeric, trades integer)
AS $$ SELECT DISTINCT
    bucket as start_time,
    bucket + intvl - interval '1 microsecond' as end_time,
    sum(entryprice) over w as usd_volume,
    sum(positionsize) over w as btc_volume,
    count(*) over w as trades
FROM (
    select date_trunc(trunc_by, timestamp) as bucket,*
    from trader_order
    WHERE timestamp BETWEEN since AND now()
) as t
WINDOW w as (partition by bucket ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
$$
LANGUAGE SQL;


CREATE FUNCTION get_candles_interval(intvl interval, trunc_by text, since timestamptz)
RETURNS TABLE(
    start_time timestamptz,
    end_time timestamptz,
    usd_volume numeric,
    btc_volume numeric,
    trades integer,
    open numeric,
    high numeric,
    low numeric,
    close numeric
)
AS $$
WITH t1 AS (
    SELECT * FROM get_ohlc_interval(intvl, trunc_by, since)
), t2 AS (
    SELECT * FROM get_volume_interval(intvl, trunc_by, since)
) SELECT
    coalesce(t1.start_time, t2.start_time) as start_time,
    coalesce(t1.end_time, t2.end_time) as start_time,
    coalesce(t2.usd_volume, 0) as usd_volume,
    coalesce(t2.btc_volume, 0) as btc_volume,
    coalesce(t2.trades, 0) as trades,
    coalesce(t1.open, 0) as open,
    coalesce(t1.high, 0) as high,
    coalesce(t1.low, 0) as low,
    coalesce(t1.close, 0) as close
FROM
    t1 FULL OUTER JOIN t2
    ON t1.start_time = t2.start_time
$$
LANGUAGE SQL;

CREATE FUNCTION update_candles_1min()
RETURNS void
AS $$ INSERT INTO candles_1min (
    start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close
)
SELECT * FROM get_candles_interval('1 minute', 'minute', (SELECT coalesce(max(start_time) - interval '10 minute', '1970-01-01 00:00:00.0000+00'::timestamptz) FROM candles_1min))
    ON CONFLICT(start_time)
    DO UPDATE SET
    start_time = excluded.start_time,
    end_time = excluded.end_time,
    usd_volume = excluded.usd_volume,
    btc_volume = excluded.btc_volume,
    trades = excluded.trades,
    open = excluded.open,
    high = excluded.high,
    low = excluded.low,
    close = excluded.close
    ;

$$
LANGUAGE SQL;

CREATE FUNCTION update_candles_1hour()
RETURNS void
AS $$ INSERT INTO candles_1hour (
    start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close
)
SELECT * FROM get_candles_interval('1 hour', 'hour', (SELECT coalesce(max(start_time) - interval '10 minute', '1970-01-01 00:00:00.0000+00'::timestamptz) FROM candles_1hour))
    ON CONFLICT(start_time)
    DO UPDATE SET
    start_time = excluded.start_time,
    end_time = excluded.end_time,
    usd_volume = excluded.usd_volume,
    btc_volume = excluded.btc_volume,
    trades = excluded.trades,
    open = excluded.open,
    high = excluded.high,
    low = excluded.low,
    close = excluded.close
    ;

$$
LANGUAGE SQL;

CREATE FUNCTION update_candles_1day()
RETURNS void
AS $$ INSERT INTO candles_1day (
    start_time, end_time, usd_volume, btc_volume, trades, open, high, low, close
)
SELECT * FROM get_candles_interval('1 day', 'day', (SELECT coalesce(max(start_time) - interval '10 minute', '1970-01-01 00:00:00.0000+00'::timestamptz) FROM candles_1day))
    ON CONFLICT(start_time)
    DO UPDATE SET
    start_time = excluded.start_time,
    end_time = excluded.end_time,
    usd_volume = excluded.usd_volume,
    btc_volume = excluded.btc_volume,
    trades = excluded.trades,
    open = excluded.open,
    high = excluded.high,
    low = excluded.low,
    close = excluded.close
    ;

$$
LANGUAGE SQL;
