use crate::database::{
    schema::{btc_usd_price, funding_rate, lend_order, trader_order},
    sql_types::*,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::sql_types::*;
use serde::{Deserialize, Serialize};
use twilight_relayer_rust::relayer;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = btc_usd_price)]
pub struct BtcUsdPrice {
    pub id: usize,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = funding_rate)]
pub struct FundingRate {
    pub id: usize,
    pub rate: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = trader_order)]
pub struct TraderOrder {
    pub uuid: Uuid,
    pub account_id: String,
    pub position_type: PositionType,
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub entryprice: BigDecimal,
    pub execution_price: BigDecimal,
    pub positionsize: BigDecimal,
    pub leverage: BigDecimal,
    pub initial_margin: BigDecimal,
    pub available_margin: BigDecimal,
    pub timestamp: DateTime<Utc>,
    pub bankruptcy_price: BigDecimal,
    pub bankruptcy_value: BigDecimal,
    pub maintenance_margin: BigDecimal,
    pub liquidation_price: BigDecimal,
    pub unrealized_pnl: BigDecimal,
    pub settlement_price: BigDecimal,
    pub entry_nonce: i64,
    pub exit_nonce: i64,
    pub entry_sequence: i64,
}

impl TraderOrder {
    pub fn insert(self, conn: &mut PgConnection) -> QueryResult<usize> {
        use crate::database::schema::trader_order::dsl::*;

        diesel::insert_into(trader_order)
            .values(&self)
            .execute(conn)
    }

    pub fn update(self, conn: &mut PgConnection) -> QueryResult<usize> {
        use crate::database::schema::trader_order::dsl::*;

        diesel::update(trader_order)
            .filter(uuid.eq(self.uuid))
            .set(&self)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = lend_order)]
pub struct LendOrder {
    pub uuid: Uuid,
    pub account_id: String,
    pub balance: f64,
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub entry_nonce: usize,
    pub exit_nonce: usize,
    pub deposit: f64,
    pub new_lend_state_amount: f64,
    pub timestamp: DateTime<Utc>,
    pub npoolshare: f64,
    pub nwithdraw: f64,
    pub payment: f64,
    pub tlv0: f64,
    pub tps0: f64,
    pub tlv1: f64,
    pub tps1: f64,
    pub tlv2: f64,
    pub tps2: f64,
    pub tlv3: f64,
    pub tps3: f64,
    pub entry_sequence: usize,
}

impl From<relayer::TraderOrder> for TraderOrder {
    fn from(src: relayer::TraderOrder) -> TraderOrder {
        let relayer::TraderOrder {
            uuid,
            account_id,
            position_type,
            order_status,
            order_type,
            entryprice,
            execution_price,
            positionsize,
            leverage,
            initial_margin,
            available_margin,
            timestamp,
            bankruptcy_price,
            bankruptcy_value,
            maintenance_margin,
            liquidation_price,
            unrealized_pnl,
            settlement_price,
            entry_nonce,
            exit_nonce,
            entry_sequence,
        } = src;

        TraderOrder {
            uuid: Uuid::from_bytes(*uuid.as_bytes()),
            account_id,
            position_type: position_type.into(),
            order_status: order_status.into(),
            order_type: order_type.into(),
            // TODO: maybe a TryFrom impl instead...
            entryprice: BigDecimal::from_f64(entryprice).unwrap(),
            execution_price: BigDecimal::from_f64(execution_price).unwrap(),
            positionsize: BigDecimal::from_f64(positionsize).unwrap(),
            leverage: BigDecimal::from_f64(leverage).unwrap(),
            initial_margin: BigDecimal::from_f64(initial_margin).unwrap(),
            available_margin: BigDecimal::from_f64(available_margin).unwrap(),
            timestamp: timestamp.into(),
            bankruptcy_price: BigDecimal::from_f64(bankruptcy_price).unwrap(),
            bankruptcy_value: BigDecimal::from_f64(bankruptcy_value).unwrap(),
            maintenance_margin: BigDecimal::from_f64(maintenance_margin).unwrap(),
            liquidation_price: BigDecimal::from_f64(liquidation_price).unwrap(),
            unrealized_pnl: BigDecimal::from_f64(unrealized_pnl).unwrap(),
            settlement_price: BigDecimal::from_f64(settlement_price).unwrap(),
            entry_nonce: entry_nonce as i64,
            exit_nonce: exit_nonce as i64,
            entry_sequence: entry_sequence as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_type() {
        let order = LendOrder {
            uuid: Uuid::nil(),
            account_id: "fake_account".into(),
            balance: 0.0,
            order_status: OrderStatus::PENDING,
            order_type: OrderType::LEND,
            entry_nonce: 0,
            exit_nonce: 0,
            deposit: 0.0,
            new_lend_state_amount: 0.0,
            timestamp: Utc::now(),
            npoolshare: 0.0,
            nwithdraw: 0.0,
            payment: 0.0,
            tlv0: 0.0,
            tps0: 0.0,
            tlv1: 0.0,
            tps1: 0.0,
            tlv2: 0.0,
            tps2: 0.0,
            tlv3: 0.0,
            tps3: 0.0,
            entry_sequence: 0,
        };
    }
}
