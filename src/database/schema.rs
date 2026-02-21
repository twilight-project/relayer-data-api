// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "lend_pool_command_type"))]
    pub struct LendPoolCommandType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_status"))]
    pub struct OrderStatus;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_type"))]
    pub struct OrderType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "position_size_command"))]
    pub struct PositionSizeCommand;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "position_type"))]
    pub struct PositionType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "sorted_set_command_type"))]
    pub struct SortedSetCommandType;
}

diesel::table! {
    address_customer_id (id) {
        id -> Int8,
        address -> Varchar,
        customer_id -> Int8,
    }
}

diesel::table! {
    btc_usd_price (id) {
        id -> Int8,
        price -> Numeric,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    candles_1day (start_time) {
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        low -> Numeric,
        high -> Numeric,
        open -> Numeric,
        close -> Numeric,
        trades -> Int4,
        btc_volume -> Numeric,
        usd_volume -> Numeric,
    }
}

diesel::table! {
    candles_1hour (start_time) {
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        low -> Numeric,
        high -> Numeric,
        open -> Numeric,
        close -> Numeric,
        trades -> Int4,
        btc_volume -> Numeric,
        usd_volume -> Numeric,
    }
}

diesel::table! {
    candles_1min (start_time) {
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        low -> Numeric,
        high -> Numeric,
        open -> Numeric,
        close -> Numeric,
        trades -> Int4,
        btc_volume -> Numeric,
        usd_volume -> Numeric,
    }
}

diesel::table! {
    current_nonce (id) {
        id -> Int8,
        nonce -> Int8,
    }
}

diesel::table! {
    customer_account (id) {
        id -> Int8,
        customer_registration_id -> Varchar,
        username -> Varchar,
        password -> Varchar,
        created_on -> Timestamptz,
        password_hint -> Varchar,
    }
}

diesel::table! {
    customer_apikey_linking (id) {
        id -> Int8,
        customer_account_id -> Int8,
        api_key -> Varchar,
        api_salt_key -> Varchar,
        created_on -> Timestamptz,
        expires_on -> Timestamptz,
        is_active -> Bool,
        remark -> Nullable<Varchar>,
        authorities -> Nullable<Varchar>,
        limit_remaining -> Nullable<Int8>,
    }
}

diesel::table! {
    customer_order_linking (id) {
        id -> Int8,
        #[max_length = 64]
        order_id -> Varchar,
        public_key -> Varchar,
        customer_account_id -> Int8,
        order_status -> Varchar,
        created_on -> Timestamptz,
    }
}

diesel::table! {
    fee_history (id) {
        id -> Int8,
        order_filled_on_market -> Numeric,
        order_filled_on_limit -> Numeric,
        order_settled_on_market -> Numeric,
        order_settled_on_limit -> Numeric,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    funding_rate (id) {
        id -> Int8,
        rate -> Numeric,
        price -> Numeric,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OrderStatus;
    use super::sql_types::OrderType;

    lend_order (id) {
        id -> Int8,
        #[max_length = 64]
        uuid -> Varchar,
        account_id -> Varchar,
        balance -> Numeric,
        order_status -> OrderStatus,
        order_type -> OrderType,
        entry_nonce -> Int8,
        exit_nonce -> Int8,
        deposit -> Numeric,
        new_lend_state_amount -> Numeric,
        timestamp -> Timestamptz,
        npoolshare -> Numeric,
        nwithdraw -> Numeric,
        payment -> Numeric,
        tlv0 -> Numeric,
        tps0 -> Numeric,
        tlv1 -> Numeric,
        tps1 -> Numeric,
        tlv2 -> Numeric,
        tps2 -> Numeric,
        tlv3 -> Numeric,
        tps3 -> Numeric,
        entry_sequence -> Int8,
    }
}

diesel::table! {
    lend_pool (id) {
        id -> Int8,
        sequence -> Int8,
        nonce -> Int8,
        total_pool_share -> Numeric,
        total_locked_value -> Numeric,
        pending_orders -> Int8,
        aggregate_log_sequence -> Int8,
        last_snapshot_id -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LendPoolCommandType;

    lend_pool_command (id) {
        id -> Int8,
        command -> LendPoolCommandType,
        #[max_length = 64]
        order_id -> Varchar,
        payment -> Nullable<Numeric>,
    }
}

diesel::table! {
    lend_pool_price_minute (bucket_ts) {
        bucket_ts -> Timestamptz,
        share_price -> Numeric,
        total_locked_value -> Numeric,
        total_pool_share -> Numeric,
        samples -> Int4,
        source -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PositionType;

    risk_engine_update (id) {
        id -> Int8,
        command -> Varchar,
        position_type -> Nullable<PositionType>,
        amount -> Nullable<Float8>,
        total_long_btc -> Float8,
        total_short_btc -> Float8,
        total_pending_long_btc -> Float8,
        total_pending_short_btc -> Float8,
        manual_halt -> Bool,
        manual_close_only -> Bool,
        pause_funding -> Bool,
        pause_price_feed -> Bool,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    risk_params_update (id) {
        id -> Int8,
        max_oi_mult -> Float8,
        max_net_mult -> Float8,
        max_position_pct -> Float8,
        min_position_btc -> Float8,
        max_leverage -> Float8,
        timestamp -> Timestamptz,
        mm_ratio -> Float8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PositionSizeCommand;
    use super::sql_types::PositionType;

    position_size_log (id) {
        id -> Int8,
        command -> PositionSizeCommand,
        position_type -> PositionType,
        amount -> Numeric,
        total_short -> Numeric,
        total_long -> Numeric,
        total -> Numeric,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SortedSetCommandType;
    use super::sql_types::PositionType;

    sorted_set_command (id) {
        id -> Int8,
        command -> SortedSetCommandType,
        uuid -> Nullable<Varchar>,
        amount -> Nullable<Numeric>,
        position_type -> PositionType,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PositionType;
    use super::sql_types::OrderStatus;
    use super::sql_types::OrderType;

    trader_order (id) {
        id -> Int8,
        #[max_length = 64]
        uuid -> Varchar,
        account_id -> Varchar,
        position_type -> PositionType,
        order_status -> OrderStatus,
        order_type -> OrderType,
        entryprice -> Numeric,
        execution_price -> Numeric,
        positionsize -> Numeric,
        leverage -> Numeric,
        initial_margin -> Numeric,
        available_margin -> Numeric,
        timestamp -> Timestamptz,
        bankruptcy_price -> Numeric,
        bankruptcy_value -> Numeric,
        maintenance_margin -> Numeric,
        liquidation_price -> Numeric,
        unrealized_pnl -> Numeric,
        settlement_price -> Numeric,
        entry_nonce -> Int8,
        exit_nonce -> Int8,
        entry_sequence -> Int8,
        fee_filled -> Numeric,
        fee_settled -> Numeric,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PositionType;
    use super::sql_types::OrderStatus;
    use super::sql_types::OrderType;

    trader_order_funding_updated (id) {
        id -> Int8,
        #[max_length = 64]
        uuid -> Varchar,
        account_id -> Varchar,
        position_type -> PositionType,
        order_status -> OrderStatus,
        order_type -> OrderType,
        entryprice -> Numeric,
        execution_price -> Numeric,
        positionsize -> Numeric,
        leverage -> Numeric,
        initial_margin -> Numeric,
        available_margin -> Numeric,
        timestamp -> Timestamptz,
        bankruptcy_price -> Numeric,
        bankruptcy_value -> Numeric,
        maintenance_margin -> Numeric,
        liquidation_price -> Numeric,
        unrealized_pnl -> Numeric,
        settlement_price -> Numeric,
        entry_nonce -> Int8,
        exit_nonce -> Int8,
        entry_sequence -> Int8,
        fee_filled -> Numeric,
        fee_settled -> Numeric,
    }
}

diesel::table! {
    twilight_qq_account_link (id) {
        id -> Int8,
        twilight_address -> Varchar,
        account_address -> Varchar,
        order_id -> Varchar,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OrderType;
    use super::sql_types::OrderStatus;

    transaction_hash (id) {
        id -> Int8,
        order_id -> Varchar,
        account_id -> Varchar,
        tx_hash -> Varchar,
        order_type -> OrderType,
        order_status -> OrderStatus,
        datetime -> Varchar,
        output -> Nullable<Varchar>,
        request_id -> Nullable<Varchar>,
    }
}

diesel::joinable!(address_customer_id -> customer_account (customer_id));
diesel::joinable!(customer_apikey_linking -> customer_account (customer_account_id));
diesel::joinable!(customer_order_linking -> customer_account (customer_account_id));

diesel::allow_tables_to_appear_in_same_query!(
    address_customer_id,
    btc_usd_price,
    candles_1day,
    candles_1hour,
    candles_1min,
    current_nonce,
    customer_account,
    customer_apikey_linking,
    customer_order_linking,
    fee_history,
    funding_rate,
    lend_order,
    lend_pool,
    lend_pool_command,
    lend_pool_price_minute,
    position_size_log,
    risk_engine_update,
    risk_params_update,
    sorted_set_command,
    trader_order,
    trader_order_funding_updated,
    transaction_hash,
    twilight_qq_account_link,
);
