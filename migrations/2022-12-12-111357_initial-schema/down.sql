-- This file should undo anything in `up.sql`
DROP TABLE lend_order;
DROP TABLE trader_order;
DROP TABLE funding_rate;
DROP TABLE btc_usd_price;
DROP TABLE position_size_log;
DROP TABLE sorted_set_command;
DROP TABLE lend_pool_command;

DROP TYPE order_status;
DROP TYPE position_type;
DROP TYPE order_type;
DROP TYPE tx_type;
DROP TYPE position_size_command;
DROP TYPE sorted_set_command_type;
DROP TYPE lend_pool_command_type;
