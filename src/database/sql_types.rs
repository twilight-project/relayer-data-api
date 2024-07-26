#![allow(non_camel_case_types)]
#![allow(warnings)]
use crate::database::schema::sql_types::{
    LendPoolCommandType as LendPoolCommandTypeSql, OrderStatus as OrderStatusSql,
    OrderType as OrderTypeSql, PositionSizeCommand as PositionSizeCommandSql,
    PositionType as PositionTypeSql, SortedSetCommandType as SortedSetCommandTypeSql,
};
use diesel::*;
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
};
use relayerwalletlib::zkoswalletlib::relayer_types;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TXType {
    ORDERTX,
    LENDTX,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = LendPoolCommandTypeSql)]
pub enum LendPoolCommandType {
    ADD_TRADER_ORDER_SETTLEMENT,
    ADD_TRADER_LIMIT_ORDER_SETTLEMENT,
    ADD_FUNDING_DATA,
    ADD_TRADER_ORDER_LIQUIDATION,
    LEND_ORDER_CREATE_ORDER,
    LEND_ORDER_SETTLE_ORDER,
    BATCH_EXECUTE_TRADER_ORDER,
    INITIATE_NEW_POOL,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = SortedSetCommandTypeSql)]
pub enum SortedSetCommandType {
    ADD_LIQUIDATION_PRICE,
    ADD_OPEN_LIMIT_PRICE,
    ADD_CLOSE_LIMIT_PRICE,
    REMOVE_LIQUIDATION_PRICE,
    REMOVE_OPEN_LIMIT_PRICE,
    REMOVE_CLOSE_LIMIT_PRICE,
    UPDATE_LIQUIDATION_PRICE,
    UPDATE_OPEN_LIMIT_PRICE,
    UPDATE_CLOSE_LIMIT_PRICE,
    BULK_SEARCH_REMOVE_LIQUIDATION_PRICE,
    BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE,
    BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = PositionSizeCommandSql)]
pub enum PositionSizeCommand {
    ADD,
    REMOVE,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression, QueryId)]
#[diesel(sql_type = OrderTypeSql)]
pub enum OrderType {
    LIMIT,
    MARKET,
    DARK,
    LEND,
}

impl diesel::query_builder::QueryId for OrderTypeSql {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = PositionTypeSql)]
pub enum PositionType {
    LONG,
    SHORT,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression, QueryId)]
#[diesel(sql_type = OrderStatusSql)]
pub enum OrderStatus {
    SETTLED,
    LENDED,
    LIQUIDATE,
    CANCELLED,
    PENDING,
    FILLED,
    DuplicateOrder,
    UtxoError,
    Error,
    NoResponseFromChain,
    BincodeError,
    HexCodeError,
    SerializationError,
    RequestSubmitted,
    OrderNotFound,
    RejectedFromChain,
    FilledUpdated,
}

impl OrderStatus {
    pub fn is_cancelable(&self) -> bool {
        use OrderStatus::*;

        match self {
            PENDING => true,
            _ => false,
        }
    }

    pub fn is_closed(&self) -> bool {
        use OrderStatus::*;

        match self {
            FILLED => true,
            _ => false,
        }
    }
}

impl diesel::query_builder::QueryId for OrderStatusSql {
    type QueryId = ();
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl ToSql<LendPoolCommandTypeSql, Pg> for LendPoolCommandType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            LendPoolCommandType::ADD_TRADER_ORDER_SETTLEMENT => {
                out.write_all(b"ADD_TRADER_ORDER_SETTLEMENT")?
            }
            LendPoolCommandType::ADD_TRADER_LIMIT_ORDER_SETTLEMENT => {
                out.write_all(b"ADD_TRADER_LIMIT_ORDER_SETTLEMENT")?
            }
            LendPoolCommandType::ADD_FUNDING_DATA => out.write_all(b"ADD_FUNDING_DATA")?,
            LendPoolCommandType::ADD_TRADER_ORDER_LIQUIDATION => {
                out.write_all(b"ADD_TRADER_ORDER_LIQUIDATION")?
            }
            LendPoolCommandType::LEND_ORDER_CREATE_ORDER => {
                out.write_all(b"LEND_ORDER_CREATE_ORDER")?
            }
            LendPoolCommandType::LEND_ORDER_SETTLE_ORDER => {
                out.write_all(b"LEND_ORDER_SETTLE_ORDER")?
            }
            LendPoolCommandType::BATCH_EXECUTE_TRADER_ORDER => {
                out.write_all(b"BATCH_EXECUTE_TRADER_ORDER")?
            }
            LendPoolCommandType::INITIATE_NEW_POOL => out.write_all(b"INITIATE_NEW_POOL")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<LendPoolCommandTypeSql, Pg> for LendPoolCommandType {
    fn from_sql(bytes: backend::RawValue<Pg>) -> deserialize::Result<LendPoolCommandType> {
        match bytes.as_bytes() {
            b"ADD_TRADER_ORDER_SETTLEMENT" => Ok(LendPoolCommandType::ADD_TRADER_ORDER_SETTLEMENT),
            b"ADD_TRADER_LIMIT_ORDER_SETTLEMENT" => {
                Ok(LendPoolCommandType::ADD_TRADER_LIMIT_ORDER_SETTLEMENT)
            }
            b"ADD_FUNDING_DATA" => Ok(LendPoolCommandType::ADD_FUNDING_DATA),
            b"ADD_TRADER_ORDER_LIQUIDATION" => {
                Ok(LendPoolCommandType::ADD_TRADER_ORDER_LIQUIDATION)
            }
            b"LEND_ORDER_CREATE_ORDER" => Ok(LendPoolCommandType::LEND_ORDER_CREATE_ORDER),
            b"LEND_ORDER_SETTLE_ORDER" => Ok(LendPoolCommandType::LEND_ORDER_SETTLE_ORDER),
            b"BATCH_EXECUTE_TRADER_ORDER" => Ok(LendPoolCommandType::BATCH_EXECUTE_TRADER_ORDER),
            b"INITIATE_NEW_POOL" => Ok(LendPoolCommandType::INITIATE_NEW_POOL),
            _ => panic!("Invalid enum type in database!"),
        }
    }
}

impl ToSql<SortedSetCommandTypeSql, Pg> for SortedSetCommandType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            SortedSetCommandType::ADD_LIQUIDATION_PRICE => {
                out.write_all(b"ADD_LIQUIDATION_PRICE")?
            }
            SortedSetCommandType::ADD_OPEN_LIMIT_PRICE => out.write_all(b"ADD_OPEN_LIMIT_PRICE")?,
            SortedSetCommandType::ADD_CLOSE_LIMIT_PRICE => {
                out.write_all(b"ADD_CLOSE_LIMIT_PRICE")?
            }
            SortedSetCommandType::REMOVE_LIQUIDATION_PRICE => {
                out.write_all(b"REMOVE_LIQUIDATION_PRICE")?
            }
            SortedSetCommandType::REMOVE_OPEN_LIMIT_PRICE => {
                out.write_all(b"REMOVE_OPEN_LIMIT_PRICE")?
            }
            SortedSetCommandType::REMOVE_CLOSE_LIMIT_PRICE => {
                out.write_all(b"REMOVE_CLOSE_LIMIT_PRICE")?
            }
            SortedSetCommandType::UPDATE_LIQUIDATION_PRICE => {
                out.write_all(b"UPDATE_LIQUIDATION_PRICE")?
            }
            SortedSetCommandType::UPDATE_OPEN_LIMIT_PRICE => {
                out.write_all(b"UPDATE_OPEN_LIMIT_PRICE")?
            }
            SortedSetCommandType::UPDATE_CLOSE_LIMIT_PRICE => {
                out.write_all(b"UPDATE_CLOSE_LIMIT_PRICE")?
            }
            SortedSetCommandType::BULK_SEARCH_REMOVE_LIQUIDATION_PRICE => {
                out.write_all(b"BULK_SEARCH_REMOVE_LIQUIDATION_PRICE")?
            }
            SortedSetCommandType::BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE => {
                out.write_all(b"BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE")?
            }
            SortedSetCommandType::BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE => {
                out.write_all(b"BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE")?
            }
        }
        Ok(IsNull::No)
    }
}

impl FromSql<SortedSetCommandTypeSql, Pg> for SortedSetCommandType {
    fn from_sql(bytes: backend::RawValue<Pg>) -> deserialize::Result<SortedSetCommandType> {
        match bytes.as_bytes() {
            b"ADD_LIQUIDATION_PRICE" => Ok(SortedSetCommandType::ADD_LIQUIDATION_PRICE),
            b"ADD_OPEN_LIMIT_PRICE" => Ok(SortedSetCommandType::ADD_OPEN_LIMIT_PRICE),
            b"ADD_CLOSE_LIMIT_PRICE" => Ok(SortedSetCommandType::ADD_CLOSE_LIMIT_PRICE),
            b"REMOVE_LIQUIDATION_PRICE" => Ok(SortedSetCommandType::REMOVE_LIQUIDATION_PRICE),
            b"REMOVE_OPEN_LIMIT_PRICE" => Ok(SortedSetCommandType::REMOVE_OPEN_LIMIT_PRICE),
            b"REMOVE_CLOSE_LIMIT_PRICE" => Ok(SortedSetCommandType::REMOVE_CLOSE_LIMIT_PRICE),
            b"UPDATE_LIQUIDATION_PRICE" => Ok(SortedSetCommandType::UPDATE_LIQUIDATION_PRICE),
            b"UPDATE_OPEN_LIMIT_PRICE" => Ok(SortedSetCommandType::UPDATE_OPEN_LIMIT_PRICE),
            b"UPDATE_CLOSE_LIMIT_PRICE" => Ok(SortedSetCommandType::UPDATE_CLOSE_LIMIT_PRICE),
            b"BULK_SEARCH_REMOVE_LIQUIDATION_PRICE" => {
                Ok(SortedSetCommandType::BULK_SEARCH_REMOVE_LIQUIDATION_PRICE)
            }
            b"BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE" => {
                Ok(SortedSetCommandType::BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE)
            }
            b"BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE" => {
                Ok(SortedSetCommandType::BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE)
            }
            _ => panic!("Invalid enum type in database!"),
        }
    }
}

impl ToSql<OrderStatusSql, Pg> for OrderStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            OrderStatus::SETTLED => out.write_all(b"SETTLED")?,
            OrderStatus::LENDED => out.write_all(b"LENDED")?,
            OrderStatus::LIQUIDATE => out.write_all(b"LIQUIDATE")?,
            OrderStatus::CANCELLED => out.write_all(b"CANCELLED")?,
            OrderStatus::PENDING => out.write_all(b"PENDING")?,
            OrderStatus::FILLED => out.write_all(b"FILLED")?,
            OrderStatus::DuplicateOrder => out.write_all(b"DuplicateOrder")?,
            OrderStatus::UtxoError => out.write_all(b"UtxoError")?,
            OrderStatus::Error => out.write_all(b"Error")?,
            OrderStatus::NoResponseFromChain => out.write_all(b"NoResponseFromChain")?,
            OrderStatus::BincodeError => out.write_all(b"BincodeError")?,
            OrderStatus::HexCodeError => out.write_all(b"HexCodeError")?,
            OrderStatus::SerializationError => out.write_all(b"SerializationError")?,
            OrderStatus::RequestSubmitted => out.write_all(b"RequestSubmitted")?,
            OrderStatus::OrderNotFound => out.write_all(b"OrderNotFound")?,
            OrderStatus::RejectedFromChain => out.write_all(b"RejectedFromChain")?,
            OrderStatus::FilledUpdated => out.write_all(b"FilledUpdated")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<PositionSizeCommandSql, Pg> for PositionSizeCommand {
    fn from_sql(bytes: backend::RawValue<Pg>) -> deserialize::Result<PositionSizeCommand> {
        match bytes.as_bytes() {
            b"ADD" => Ok(PositionSizeCommand::ADD),
            b"REMOVE" => Ok(PositionSizeCommand::REMOVE),
            _ => panic!("Invalid enum type in database!"),
        }
    }
}

impl ToSql<PositionSizeCommandSql, Pg> for PositionSizeCommand {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PositionSizeCommand::ADD => out.write_all(b"ADD")?,
            PositionSizeCommand::REMOVE => out.write_all(b"REMOVE")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<OrderStatusSql, Pg> for OrderStatus {
    fn from_sql(bytes: backend::RawValue<Pg>) -> deserialize::Result<OrderStatus> {
        match bytes.as_bytes() {
            b"SETTLED" => Ok(OrderStatus::SETTLED),
            b"LENDED" => Ok(OrderStatus::LENDED),
            b"LIQUIDATE" => Ok(OrderStatus::LIQUIDATE),
            b"CANCELLED" => Ok(OrderStatus::CANCELLED),
            b"PENDING" => Ok(OrderStatus::PENDING),
            b"FILLED" => Ok(OrderStatus::FILLED),
            b"DuplicateOrder" => Ok(OrderStatus::DuplicateOrder),
            b"UtxoError" => Ok(OrderStatus::UtxoError),
            b"Error" => Ok(OrderStatus::Error),
            b"NoResponseFromChain" => Ok(OrderStatus::NoResponseFromChain),
            b"BincodeError" => Ok(OrderStatus::BincodeError),
            b"HexCodeError" => Ok(OrderStatus::HexCodeError),
            b"SerializationError" => Ok(OrderStatus::SerializationError),
            b"RequestSubmitted" => Ok(OrderStatus::RequestSubmitted),
            b"OrderNotFound" => Ok(OrderStatus::OrderNotFound),
            b"RejectedFromChain" => Ok(OrderStatus::RejectedFromChain),
            b"FilledUpdated" => Ok(OrderStatus::FilledUpdated),
            _ => panic!("Invalid enum type in database!"),
        }
    }
}

impl ToSql<OrderTypeSql, Pg> for OrderType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            OrderType::LIMIT => out.write_all(b"LIMIT")?,
            OrderType::MARKET => out.write_all(b"MARKET")?,
            OrderType::DARK => out.write_all(b"DARK")?,
            OrderType::LEND => out.write_all(b"LEND")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<OrderTypeSql, Pg> for OrderType {
    fn from_sql(bytes: backend::RawValue<Pg>) -> deserialize::Result<OrderType> {
        match bytes.as_bytes() {
            b"LIMIT" => Ok(OrderType::LIMIT),
            b"MARKET" => Ok(OrderType::MARKET),
            b"DARK" => Ok(OrderType::DARK),
            b"LEND" => Ok(OrderType::LEND),
            _ => panic!("Invalid enum type in database!"),
        }
    }
}

impl ToSql<PositionTypeSql, Pg> for PositionType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PositionType::LONG => out.write_all(b"LONG")?,
            PositionType::SHORT => out.write_all(b"SHORT")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<PositionTypeSql, Pg> for PositionType {
    fn from_sql(bytes: backend::RawValue<Pg>) -> deserialize::Result<PositionType> {
        match bytes.as_bytes() {
            b"LONG" => Ok(PositionType::LONG),
            b"SHORT" => Ok(PositionType::SHORT),
            _ => panic!("Invalid enum type in database!"),
        }
    }
}

impl From<relayer_types::OrderStatus> for OrderStatus {
    fn from(status: relayer_types::OrderStatus) -> OrderStatus {
        match status {
            relayer_types::OrderStatus::SETTLED => OrderStatus::SETTLED,
            relayer_types::OrderStatus::LENDED => OrderStatus::LENDED,
            relayer_types::OrderStatus::LIQUIDATE => OrderStatus::LIQUIDATE,
            relayer_types::OrderStatus::CANCELLED => OrderStatus::CANCELLED,
            relayer_types::OrderStatus::PENDING => OrderStatus::PENDING,
            relayer_types::OrderStatus::FILLED => OrderStatus::FILLED,
            relayer_types::OrderStatus::DuplicateOrder => OrderStatus::DuplicateOrder,
            relayer_types::OrderStatus::UtxoError => OrderStatus::UtxoError,
            relayer_types::OrderStatus::Error => OrderStatus::Error,
            relayer_types::OrderStatus::NoResponseFromChain => OrderStatus::NoResponseFromChain,
            relayer_types::OrderStatus::BincodeError => OrderStatus::BincodeError,
            relayer_types::OrderStatus::HexCodeError => OrderStatus::HexCodeError,
            relayer_types::OrderStatus::SerializationError => OrderStatus::SerializationError,
            relayer_types::OrderStatus::RequestSubmitted => OrderStatus::RequestSubmitted,
            relayer_types::OrderStatus::OrderNotFound => OrderStatus::OrderNotFound,
            relayer_types::OrderStatus::RejectedFromChain => OrderStatus::RejectedFromChain,
            relayer_types::OrderStatus::FilledUpdated => OrderStatus::FilledUpdated,
        }
    }
}

impl From<relayer_types::OrderType> for OrderType {
    fn from(typ: relayer_types::OrderType) -> OrderType {
        match typ {
            relayer_types::OrderType::LIMIT => OrderType::LIMIT,
            relayer_types::OrderType::MARKET => OrderType::MARKET,
            relayer_types::OrderType::DARK => OrderType::DARK,
            relayer_types::OrderType::LEND => OrderType::LEND,
        }
    }
}

impl From<relayer_types::PositionType> for PositionType {
    fn from(typ: relayer_types::PositionType) -> PositionType {
        match typ {
            relayer_types::PositionType::LONG => PositionType::LONG,
            relayer_types::PositionType::SHORT => PositionType::SHORT,
        }
    }
}

impl From<relayer_types::TXType> for TXType {
    fn from(typ: relayer_types::TXType) -> TXType {
        match typ {
            relayer_types::TXType::ORDERTX => TXType::ORDERTX,
            relayer_types::TXType::LENDTX => TXType::LENDTX,
        }
    }
}
