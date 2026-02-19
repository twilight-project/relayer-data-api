-- Add pause_funding and pause_price_feed columns to risk_engine_update,
-- corresponding to the two new RiskEngineCommand variants added in
-- relayer-core branch risk-param-fix-halt.
ALTER TABLE risk_engine_update
    ADD COLUMN pause_funding    BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN pause_price_feed BOOLEAN NOT NULL DEFAULT FALSE;
