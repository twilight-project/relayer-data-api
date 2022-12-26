use crate::database::{
    schema::{btc_usd_price, funding_rate, lend_order, trader_order},
    sql_types::*,
};
use chrono::prelude::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = trader_order)]
pub struct TraderOrder {
    pub uuid: Uuid,
    pub account_id: String,
    pub position_type: PositionType,
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub entryprice: f64,
    pub execution_price: f64,
    pub positionsize: f64,
    pub leverage: f64,
    pub initial_margin: f64,
    pub available_margin: f64,
    pub timestamp: DateTime<Utc>,
    pub bankruptcy_price: f64,
    pub bankruptcy_value: f64,
    pub maintenance_margin: f64,
    pub liquidation_price: f64,
    pub unrealized_pnl: f64,
    pub settlement_price: f64,
    pub entry_nonce: usize,
    pub exit_nonce: usize,
    pub entry_sequence: usize,
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
