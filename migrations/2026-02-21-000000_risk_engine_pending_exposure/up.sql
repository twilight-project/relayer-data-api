ALTER TABLE risk_engine_update
    ADD COLUMN total_pending_long_btc  DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN total_pending_short_btc DOUBLE PRECISION NOT NULL DEFAULT 0.0;
