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
    btc_usd_price (id) {
        id -> Int8,
        price -> Numeric,
        timestamp -> Timestamptz,
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
        order_id -> Varchar,
        public_key -> Varchar,
        customer_account_id -> Int8,
        order_status -> Varchar,
        created_on -> Timestamptz,
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
        order_id -> Varchar,
        payment -> Nullable<Numeric>,
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
        uuid -> Nullable<Uuid>,
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
    }
}

diesel::joinable!(customer_apikey_linking -> customer_account (customer_account_id));
diesel::joinable!(customer_order_linking -> customer_account (customer_account_id));

diesel::allow_tables_to_appear_in_same_query!(
    btc_usd_price,
    current_nonce,
    customer_account,
    customer_apikey_linking,
    customer_order_linking,
    funding_rate,
    lend_order,
    lend_pool,
    lend_pool_command,
    position_size_log,
    sorted_set_command,
    trader_order,
);
