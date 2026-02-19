CREATE TABLE twilight_qq_account_link (
    id BIGSERIAL PRIMARY KEY,
    twilight_address VARCHAR NOT NULL,
    account_address VARCHAR NOT NULL,
    order_id VARCHAR NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_twilight_qq_account_link_timestamp ON twilight_qq_account_link (timestamp DESC);
