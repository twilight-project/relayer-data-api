-- Drop triggers first (they depend on the trigger function)
DROP TRIGGER IF EXISTS lend_pool_price_minute_ins_trg ON lend_pool;
DROP TRIGGER IF EXISTS lend_pool_price_minute_upd_trg ON lend_pool;

-- Drop helper functions
DROP FUNCTION IF EXISTS last_day_apy_now();
DROP FUNCTION IF EXISTS apy_series(interval, interval, interval);

-- Drop trigger function
DROP FUNCTION IF EXISTS upsert_lend_pool_price_minute();

-- Drop table last
DROP TABLE IF EXISTS lend_pool_price_minute;
