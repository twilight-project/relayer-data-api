-- This file should undo anything in `up.sql`
-- DROP INDEX trader_order_time;
-- DROP INDEX price_time;
DROP INDEX trader_order_uuid;

DROP TABLE candles_1min;
DROP TABLE candles_1hour;
DROP TABLE candles_1day;

DROP FUNCTION get_ohlc_interval;
DROP FUNCTION get_volume_interval;
DROP FUNCTION get_candles_interval;
DROP FUNCTION update_candles_1min;
DROP FUNCTION update_candles_1hour;
DROP FUNCTION update_candles_1day;
