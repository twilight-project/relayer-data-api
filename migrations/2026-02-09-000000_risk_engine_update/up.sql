CREATE TABLE risk_engine_update (
    id BIGSERIAL PRIMARY KEY,
    command VARCHAR NOT NULL,
    position_type position_type,
    amount DOUBLE PRECISION,
    total_long_btc DOUBLE PRECISION NOT NULL,
    total_short_btc DOUBLE PRECISION NOT NULL,
    manual_halt BOOLEAN NOT NULL,
    manual_close_only BOOLEAN NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_risk_engine_update_timestamp ON risk_engine_update (timestamp DESC);
