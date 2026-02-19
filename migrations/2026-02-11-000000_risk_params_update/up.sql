CREATE TABLE risk_params_update (
    id BIGSERIAL PRIMARY KEY,
    max_oi_mult DOUBLE PRECISION NOT NULL,
    max_net_mult DOUBLE PRECISION NOT NULL,
    max_position_pct DOUBLE PRECISION NOT NULL,
    min_position_btc DOUBLE PRECISION NOT NULL,
    max_leverage DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_risk_params_update_timestamp ON risk_params_update (timestamp DESC);
