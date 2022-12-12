-- This file should undo anything in `up.sql`
DROP TABLE lend_order;
DROP TABLE trader_order;
DROP TABLE funding_rate;
DROP TABLE btc_usd_price;

DROP TYPE order_status;
DROP TYPE position_type;
DROP TYPE order_type;
DROP TYPE tx_type;
