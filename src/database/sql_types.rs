use diesel::*;
use diesel::{
    backend::Backend,
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use twilight_relayer_rust::relayer;
use crate::database::schema::sql_types::{
    OrderStatus as OrderStatusSql,
    OrderType as OrderTypeSql,
//    PositionType as PositionTypeSql,
};

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

//#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression)]
//#[diesel(sql_type = PositionTypeSql)]
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

//impl<DB: Backend> ToSql<PositionTypeSql, DB> for PositionType {
//    fn to_sql(&self, out: &mut Output<DB>) -> serialize::Result {
//        match *self {
//            PositionType::LONG => out.write_all(b"LONG")?,
//            PositionType::SHORT => out.write_all(b"SHORT")?,
//        }
//    }
//}

//impl FromSql<PositionTypeSql, Pg> for PositionType {
//}

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
