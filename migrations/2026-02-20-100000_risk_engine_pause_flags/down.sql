ALTER TABLE risk_engine_update
    DROP COLUMN IF EXISTS pause_funding,
    DROP COLUMN IF EXISTS pause_price_feed;
