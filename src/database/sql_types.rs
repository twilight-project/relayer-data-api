use crate::database::schema::sql_types::{
    OrderStatus as OrderStatusSql, OrderType as OrderTypeSql, PositionType as PositionTypeSql,
};
use diesel::*;
use diesel::{
    backend,
    deserialize::FromSql,
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use twilight_relayer_rust::relayer;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TXType {
    ORDERTX,
    LENDTX,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = OrderTypeSql)]
pub enum OrderType {
    LIMIT,
    MARKET,
    DARK,
    LEND,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = PositionTypeSql)]
pub enum PositionType {
    LONG,
    SHORT,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = OrderStatusSql)]
pub enum OrderStatus {
    SETTLED,
    LENDED,
    LIQUIDATE,
    CANCELLED,
    PENDING,
    FILLED,
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

impl From<relayer::OrderStatus> for OrderStatus {
    fn from(status: relayer::OrderStatus) -> OrderStatus {
        match status {
            relayer::OrderStatus::SETTLED => OrderStatus::SETTLED,
            relayer::OrderStatus::LENDED => OrderStatus::LENDED,
            relayer::OrderStatus::LIQUIDATE => OrderStatus::LIQUIDATE,
            relayer::OrderStatus::CANCELLED => OrderStatus::CANCELLED,
            relayer::OrderStatus::PENDING => OrderStatus::PENDING,
            relayer::OrderStatus::FILLED => OrderStatus::FILLED,
        }
    }
}

impl From<relayer::OrderType> for OrderType {
    fn from(typ: relayer::OrderType) -> OrderType {
        match typ {
            relayer::OrderType::LIMIT => OrderType::LIMIT,
            relayer::OrderType::MARKET => OrderType::MARKET,
            relayer::OrderType::DARK => OrderType::DARK,
            relayer::OrderType::LEND => OrderType::LEND,
        }
    }
}

impl From<relayer::PositionType> for PositionType {
    fn from(typ: relayer::PositionType) -> PositionType {
        match typ {
            relayer::PositionType::LONG => PositionType::LONG,
            relayer::PositionType::SHORT => PositionType::SHORT,
        }
    }
}

impl From<relayer::TXType> for TXType {
    fn from(typ: relayer::TXType) -> TXType {
        match typ {
            relayer::TXType::ORDERTX => TXType::ORDERTX,
            relayer::TXType::LENDTX => TXType::LENDTX,
        }
    }
}
