// @generated automatically by Diesel CLI.

pub mod sql_types {
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

    lend_order (uuid) {
        uuid -> Uuid,
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

    trader_order (uuid) {
        uuid -> Uuid,
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

diesel::allow_tables_to_appear_in_same_query!(
    btc_usd_price,
    funding_rate,
    lend_order,
    position_size_log,
    sorted_set_command,
    trader_order,
);
