use crate::database::{
    schema::{btc_usd_price, funding_rate, lend_order, trader_order},
    sql_types::*,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::upsert::*;
use serde::{Deserialize, Serialize};
use twilight_relayer_rust::relayer;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = btc_usd_price)]
pub struct BtcUsdPrice {
    pub id: usize,
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = btc_usd_price)]
pub struct CurrentPriceUpdate {
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

impl CurrentPriceUpdate {
    pub fn insert(conn: &mut PgConnection, current_price: f64) -> QueryResult<usize> {
        use crate::database::schema::btc_usd_price::dsl::*;

        let update = CurrentPriceUpdate {
            price: BigDecimal::from_f64(current_price).unwrap(),
            timestamp: Utc::now(),
        };

        diesel::insert_into(btc_usd_price)
            .values(update)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = funding_rate)]
pub struct FundingRate {
    pub id: usize,
    pub rate: BigDecimal,
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = funding_rate)]
pub struct FundingRateUpdate {
    pub rate: BigDecimal,
    // TODO: where to get this from????
    //pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

impl FundingRateUpdate {
    pub fn insert(conn: &mut PgConnection, r: f64) -> QueryResult<usize> {
        use crate::database::schema::funding_rate::dsl::*;

        let update = FundingRateUpdate {
            rate: BigDecimal::from_f64(r).unwrap(),
            timestamp: Utc::now(),
        };

        diesel::insert_into(funding_rate)
            .values(update)
            .execute(conn)
    }
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
    pub fn update_or_insert(
        conn: &mut PgConnection,
        orders: Vec<TraderOrder>,
    ) -> QueryResult<usize> {
        use crate::database::schema::trader_order::dsl::*;

        let query = diesel::insert_into(trader_order)
            .values(&orders)
            .on_conflict(uuid)
            .do_update()
            .set((
                account_id.eq(excluded(account_id)),
                position_type.eq(excluded(position_type)),
                order_status.eq(excluded(order_status)),
                order_type.eq(excluded(order_type)),
                entryprice.eq(excluded(entryprice)),
                execution_price.eq(excluded(execution_price)),
                positionsize.eq(excluded(positionsize)),
                leverage.eq(excluded(leverage)),
                initial_margin.eq(excluded(initial_margin)),
                available_margin.eq(excluded(available_margin)),
                timestamp.eq(excluded(timestamp)),
                bankruptcy_price.eq(excluded(bankruptcy_price)),
                bankruptcy_value.eq(excluded(bankruptcy_value)),
                maintenance_margin.eq(excluded(maintenance_margin)),
                liquidation_price.eq(excluded(liquidation_price)),
                unrealized_pnl.eq(excluded(unrealized_pnl)),
                settlement_price.eq(excluded(settlement_price)),
                entry_nonce.eq(excluded(entry_nonce)),
                exit_nonce.eq(excluded(exit_nonce)),
                entry_sequence.eq(excluded(entry_sequence)),
            ));

        query.execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = lend_order)]
pub struct LendOrder {
    pub uuid: Uuid,
    pub account_id: String,
    pub balance: BigDecimal,
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub entry_nonce: i64,
    pub exit_nonce: i64,
    pub deposit: BigDecimal,
    pub new_lend_state_amount: BigDecimal,
    pub timestamp: DateTime<Utc>,
    pub npoolshare: BigDecimal,
    pub nwithdraw: BigDecimal,
    pub payment: BigDecimal,
    pub tlv0: BigDecimal,
    pub tps0: BigDecimal,
    pub tlv1: BigDecimal,
    pub tps1: BigDecimal,
    pub tlv2: BigDecimal,
    pub tps2: BigDecimal,
    pub tlv3: BigDecimal,
    pub tps3: BigDecimal,
    pub entry_sequence: i64,
}

impl LendOrder {
    pub fn update_or_insert(conn: &mut PgConnection, orders: Vec<LendOrder>) -> QueryResult<usize> {
        use crate::database::schema::lend_order::dsl::*;

        let query = diesel::insert_into(lend_order)
            .values(&orders)
            .on_conflict(uuid)
            .do_update()
            .set((
                account_id.eq(excluded(account_id)),
                balance.eq(excluded(balance)),
                order_status.eq(excluded(order_status)),
                order_type.eq(excluded(order_type)),
                entry_nonce.eq(excluded(entry_nonce)),
                exit_nonce.eq(excluded(exit_nonce)),
                deposit.eq(excluded(deposit)),
                new_lend_state_amount.eq(excluded(new_lend_state_amount)),
                timestamp.eq(excluded(timestamp)),
                npoolshare.eq(excluded(npoolshare)),
                nwithdraw.eq(excluded(nwithdraw)),
                payment.eq(excluded(payment)),
                tlv0.eq(excluded(tlv0)),
                tps0.eq(excluded(tps0)),
                tlv1.eq(excluded(tlv1)),
                tps1.eq(excluded(tps1)),
                tlv2.eq(excluded(tlv2)),
                tps2.eq(excluded(tps2)),
                tlv3.eq(excluded(tlv3)),
                tps3.eq(excluded(tps3)),
                entry_sequence.eq(excluded(entry_sequence)),
            ));

        query.execute(conn)
    }
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

impl From<relayer::LendOrder> for LendOrder {
    fn from(src: relayer::LendOrder) -> LendOrder {
        let relayer::LendOrder {
            uuid,
            account_id,
            balance,
            order_status,
            order_type,
            entry_nonce,
            exit_nonce,
            deposit,
            new_lend_state_amount,
            timestamp,
            npoolshare,
            nwithdraw,
            payment,
            tlv0,
            tps0,
            tlv1,
            tps1,
            tlv2,
            tps2,
            tlv3,
            tps3,
            entry_sequence,
        } = src;

        LendOrder {
            uuid: Uuid::from_bytes(*uuid.as_bytes()),
            account_id,
            balance: BigDecimal::from_f64(balance).unwrap(),
            order_status: order_status.into(),
            order_type: order_type.into(),
            entry_nonce: entry_nonce as i64,
            exit_nonce: exit_nonce as i64,
            deposit: BigDecimal::from_f64(deposit).unwrap(),
            new_lend_state_amount: BigDecimal::from_f64(new_lend_state_amount).unwrap(),
            timestamp: timestamp.into(),
            npoolshare: BigDecimal::from_f64(npoolshare).unwrap(),
            nwithdraw: BigDecimal::from_f64(nwithdraw).unwrap(),
            payment: BigDecimal::from_f64(payment).unwrap(),
            tlv0: BigDecimal::from_f64(tlv0).unwrap(),
            tps0: BigDecimal::from_f64(tps0).unwrap(),
            tlv1: BigDecimal::from_f64(tlv1).unwrap(),
            tps1: BigDecimal::from_f64(tps1).unwrap(),
            tlv2: BigDecimal::from_f64(tlv2).unwrap(),
            tps2: BigDecimal::from_f64(tps2).unwrap(),
            tlv3: BigDecimal::from_f64(tlv3).unwrap(),
            tps3: BigDecimal::from_f64(tps3).unwrap(),
            entry_sequence: entry_sequence as i64,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use getrandom::getrandom;
    const DIESEL_TEST_URL: &str = "postgres://relayer:relayer@localhost:5434/test";

    fn make_trader_order(entryprice: f64, execution_price: f64) -> TraderOrder {
        let mut bytes = [0u8; 16];

        getrandom(&mut bytes).expect("Could not get randomness");

        TraderOrder {
            uuid: Uuid::from_slice(&bytes).unwrap(),
            account_id: "my-id".into(),
            position_type: PositionType::LONG,
            order_status: OrderStatus::PENDING,
            order_type: OrderType::MARKET,
            entryprice: BigDecimal::from_f64(entryprice).unwrap(),
            execution_price: BigDecimal::from_f64(execution_price).unwrap(),
            positionsize: BigDecimal::from_f64(0.0).unwrap(),
            leverage: BigDecimal::from_f64(0.0).unwrap(),
            initial_margin: BigDecimal::from_f64(0.0).unwrap(),
            available_margin: BigDecimal::from_f64(0.0).unwrap(),
            timestamp: Utc::now(),
            bankruptcy_price: BigDecimal::from_f64(0.0).unwrap(),
            bankruptcy_value: BigDecimal::from_f64(0.0).unwrap(),
            maintenance_margin: BigDecimal::from_f64(0.0).unwrap(),
            liquidation_price: BigDecimal::from_f64(0.0).unwrap(),
            unrealized_pnl: BigDecimal::from_f64(0.0).unwrap(),
            settlement_price: BigDecimal::from_f64(0.0).unwrap(),
            entry_nonce: 20,
            exit_nonce: 22,
            entry_sequence: 400,
        }
    }

    fn make_lend_order(balance: f64, payment: f64) -> LendOrder {
        let mut bytes = [0u8; 16];

        getrandom(&mut bytes).expect("Could not get randomness");

        LendOrder {
            uuid: Uuid::from_slice(&bytes).unwrap(),
            account_id: "lender-id".into(),
            balance: BigDecimal::from_f64(balance).unwrap(),
            order_status: OrderStatus::PENDING,
            order_type: OrderType::MARKET,
            entry_nonce: 40,
            exit_nonce: 600,
            deposit: BigDecimal::from_f64(0.0).unwrap(),
            new_lend_state_amount: BigDecimal::from_f64(0.0).unwrap(),
            timestamp: Utc::now(),
            npoolshare: BigDecimal::from_f64(0.0).unwrap(),
            nwithdraw: BigDecimal::from_f64(0.0).unwrap(),
            payment: BigDecimal::from_f64(payment).unwrap(),
            tlv0: BigDecimal::from_f64(0.0).unwrap(),
            tps0: BigDecimal::from_f64(0.0).unwrap(),
            tlv1: BigDecimal::from_f64(0.0).unwrap(),
            tps1: BigDecimal::from_f64(0.0).unwrap(),
            tlv2: BigDecimal::from_f64(0.0).unwrap(),
            tps2: BigDecimal::from_f64(0.0).unwrap(),
            tlv3: BigDecimal::from_f64(0.0).unwrap(),
            tps3: BigDecimal::from_f64(0.0).unwrap(),
            entry_sequence: 0,
        }
    }

    #[test]
    fn trader_orders() {
        use crate::database::schema::trader_order::dsl::*;

        let mut conn =
            PgConnection::establish(DIESEL_TEST_URL).expect("Could not establish test connection!");

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let mut order1 = make_trader_order(1.0, 4.0);
            let mut order2 = make_trader_order(4.0, 400.0);

            let orders: Vec<TraderOrder> = vec![order1.clone(), order2.clone()];

            let result = diesel::insert_into(trader_order)
                .values(orders)
                .execute(&mut *conn);

            if let Err(e) = result {
                panic!("insert in database didn't suceed! {:#?}", e);
            }

            //Test updates/inserts
            let order3 = make_trader_order(989.0, 23.0);
            let order4 = make_trader_order(99.0, 302.0);

            order1.entryprice = BigDecimal::from_f64(32.0).unwrap();
            order1.execution_price = BigDecimal::from_f64(89.0).unwrap();
            order2.entryprice = BigDecimal::from_f64(20.0).unwrap();

            TraderOrder::update_or_insert(
                &mut *conn,
                vec![
                    order1.clone(),
                    order2.clone(),
                    order3.clone(),
                    order4.clone(),
                ],
            )?;

            let o1: TraderOrder = trader_order.find(order1.uuid).first(&mut *conn)?;

            assert_eq!(o1.entryprice, order1.entryprice);
            assert_eq!(o1.execution_price, order1.execution_price);

            let o2: TraderOrder = trader_order.find(order2.uuid).first(&mut *conn)?;

            assert_eq!(o2.entryprice, order2.entryprice);
            assert_eq!(o2.execution_price, order2.execution_price);

            Ok(())
        });
    }

    #[test]
    fn lender_orders() {
        use crate::database::schema::lend_order::dsl::*;

        let mut conn =
            PgConnection::establish(DIESEL_TEST_URL).expect("Could not establish test connection!");

        conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
            let mut order1 = make_lend_order(1.0, 4.0);
            let mut order2 = make_lend_order(4.0, 400.0);

            let orders: Vec<LendOrder> = vec![order1.clone(), order2.clone()];

            let result = diesel::insert_into(lend_order)
                .values(orders)
                .execute(&mut *conn);

            if let Err(e) = result {
                panic!("insert in database didn't suceed! {:#?}", e);
            }

            //Test updates/inserts
            let order3 = make_lend_order(989.0, 23.0);
            let order4 = make_lend_order(99.0, 302.0);

            order1.balance = BigDecimal::from_f64(32.0).unwrap();
            order1.payment = BigDecimal::from_f64(89.0).unwrap();
            order2.balance = BigDecimal::from_f64(20.0).unwrap();

            LendOrder::update_or_insert(
                &mut *conn,
                vec![
                    order1.clone(),
                    order2.clone(),
                    order3.clone(),
                    order4.clone(),
                ],
            )?;

            let o1: LendOrder = lend_order.find(order1.uuid).first(&mut *conn)?;

            assert_eq!(o1.balance, order1.balance);
            assert_eq!(o1.payment, order1.payment);

            let o2: LendOrder = lend_order.find(order2.uuid).first(&mut *conn)?;

            assert_eq!(o2.balance, order2.balance);
            assert_eq!(o2.payment, order2.payment);

            Ok(())
        });
    }
}
