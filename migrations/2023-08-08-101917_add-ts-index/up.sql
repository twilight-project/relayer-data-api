-- Your SQL goes here
CREATE INDEX timestamp_btc_usd_price on btc_usd_price (timestamp);
CREATE INDEX timestamp_funding_rate on funding_rate (timestamp);
CREATE INDEX timestamp_trader_order on trader_order (timestamp);
CREATE INDEX timestamp_lend_order on lend_order (timestamp);
