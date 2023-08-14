use crate::database::{
    schema::{
        btc_usd_price, current_nonce, customer_account, customer_apikey_linking,
        customer_order_linking, funding_rate, lend_order, lend_pool, lend_pool_command,
        position_size_log, sorted_set_command, trader_order,
    },
    sql_types::*,
};
use crate::rpc::{
    HistoricalFundingArgs, HistoricalPriceArgs, Interval, OrderHistoryArgs, PnlArgs,
    TradeVolumeArgs,
};
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero};
use chrono::prelude::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use twilight_relayer_rust::{db as relayer_db, relayer};
use uuid::Uuid;

pub type PositionSizeUpdate = (relayer::PositionSizeLogCommand, relayer_db::PositionSizeLog);

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_account)]
pub struct CustomerAccount {
    id: i64,
    customer_registration_id: String,
    username: String,
    password: String,
    created_on: DateTime<Utc>,
    password_hint: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_account)]
pub struct NewCustomerAccount {
    customer_registration_id: String,
    username: String,
    password: String,
    created_on: DateTime<Utc>,
    password_hint: String,
}

impl CustomerAccount {
    pub fn get(conn: &mut PgConnection, ident: i64) -> QueryResult<CustomerAccount> {
        use crate::database::schema::customer_account::dsl::*;

        customer_account.find(ident).first(conn)
    }

    pub fn create(conn: &mut PgConnection, new_account: NewCustomerAccount) -> QueryResult<usize> {
        use crate::database::schema::customer_account::dsl::*;

        diesel::insert_into(customer_account)
            .values(new_account)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_apikey_linking)]
pub struct CustomerApiKeyLinking {
    id: i64,
    customer_account_id: i64,
    api_key: String,
    api_salt_key: String,
    created_on: DateTime<Utc>,
    expires_on: DateTime<Utc>,
    is_active: bool,
    remark: Option<String>,
    authorities: Option<String>,
    limit_remaining: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_apikey_linking)]
pub struct NewCustomerApiKeyLinking {
    customer_account_id: i64,
    api_key: String,
    api_salt_key: String,
    created_on: DateTime<Utc>,
    expires_on: DateTime<Utc>,
    is_active: bool,
    remark: Option<String>,
    authorities: Option<String>,
    limit_remaining: Option<i64>,
}

impl CustomerApiKeyLinking {
    pub fn get(conn: &mut PgConnection, ident: i64) -> QueryResult<CustomerApiKeyLinking> {
        use crate::database::schema::customer_apikey_linking::dsl::*;

        customer_apikey_linking.find(ident).first(conn)
    }

    pub fn insert(
        conn: &mut PgConnection,
        new_apikey: NewCustomerApiKeyLinking,
    ) -> QueryResult<usize> {
        use crate::database::schema::customer_apikey_linking::dsl::*;

        diesel::insert_into(customer_apikey_linking)
            .values(new_apikey)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_order_linking)]
pub struct CustomerOrderLinking {
    id: i64,
    order_id: String,
    public_key: String,
    customer_account_id: i64,
    order_status: String,
    created_on: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_order_linking)]
pub struct NewCustomerOrderLinking {
    order_id: String,
    public_key: String,
    customer_account_id: i64,
    order_status: String,
    created_on: DateTime<Utc>,
}

impl CustomerOrderLinking {
    pub fn get(conn: &mut PgConnection, ident: i64) -> QueryResult<CustomerOrderLinking> {
        use crate::database::schema::customer_order_linking::dsl::*;

        customer_order_linking.find(ident).first(conn)
    }

    pub fn insert(
        conn: &mut PgConnection,
        new_order: NewCustomerOrderLinking,
    ) -> QueryResult<usize> {
        use crate::database::schema::customer_order_linking::dsl::*;

        diesel::insert_into(customer_order_linking)
            .values(new_order)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = current_nonce)]
pub struct Nonce {
    pub id: i64,
    pub nonce: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = current_nonce)]
pub struct NewNonce {
    pub nonce: i64,
}

impl Nonce {
    pub fn get(conn: &mut PgConnection) -> QueryResult<Nonce> {
        use crate::database::schema::current_nonce::dsl::*;

        match current_nonce.order_by(id.desc()).first(conn) {
            Ok(n) => Ok(n),
            Err(diesel::result::Error::NotFound) => {
                let n = NewNonce { nonce: 0 };
                diesel::insert_into(current_nonce).values(n).execute(conn)?;

                current_nonce.order_by(id.desc()).first(conn)
            }
            e => e,
        }
    }

    pub fn increment(&mut self) -> QueryResult<()> {
        self.nonce += 1;
        Ok(())
    }

    pub fn save(&self, conn: &mut PgConnection) -> QueryResult<()> {
        use crate::database::schema::current_nonce::dsl::*;

        let _ = diesel::insert_into(current_nonce)
            .values(&*self)
            .on_conflict(id)
            .do_update()
            .set(nonce.eq(self.nonce))
            .execute(conn);

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = lend_pool)]
pub struct LendPool {
    id: i64,
    sequence: i64,
    nonce: i64,
    total_pool_share: BigDecimal,
    total_locked_value: BigDecimal,
    pending_orders: i64,
    aggregate_log_sequence: i64,
    last_snapshot_id: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = lend_pool)]
pub struct LendPoolUpdate {
    sequence: i64,
    nonce: i64,
    total_pool_share: BigDecimal,
    total_locked_value: BigDecimal,
    pending_orders: i64,
    aggregate_log_sequence: i64,
    last_snapshot_id: i64,
}

impl LendPool {
    pub fn get(conn: &mut PgConnection) -> QueryResult<LendPool> {
        use crate::database::schema::lend_pool::dsl::*;

        lend_pool.order_by(sequence.desc()).first(conn)
    }

    pub fn insert(
        conn: &mut PgConnection,
        updates: Vec<relayer_db::LendPool>,
    ) -> QueryResult<usize> {
        use crate::database::schema::lend_pool::dsl::*;

        let items: Vec<LendPoolUpdate> = updates
            .into_iter()
            .map(|update| LendPoolUpdate {
                sequence: update.sequence as i64,
                nonce: update.nonce as i64,
                total_pool_share: BigDecimal::default(),
                total_locked_value: BigDecimal::default(),
                pending_orders: 0,
                aggregate_log_sequence: update.aggrigate_log_sequence as i64,
                last_snapshot_id: update.last_snapshot_id as i64,
            })
            .collect();

        diesel::insert_into(lend_pool).values(items).execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = lend_pool_command)]
pub struct LendPoolCommand {
    id: i64,
    command: LendPoolCommandType,
    order_id: String,
    payment: Option<BigDecimal>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = lend_pool_command)]
pub struct LendPoolCommandUpdate {
    command: LendPoolCommandType,
    order_id: String,
    payment: Option<BigDecimal>,
}

impl From<(LendPoolCommandType, String, Option<BigDecimal>)> for LendPoolCommandUpdate {
    fn from(tuple: (LendPoolCommandType, String, Option<BigDecimal>)) -> LendPoolCommandUpdate {
        let (command, order_id, payment) = tuple;

        LendPoolCommandUpdate {
            command,
            order_id,
            payment,
        }
    }
}

impl LendPoolCommand {
    pub fn insert(
        conn: &mut PgConnection,
        updates: Vec<relayer_db::LendPoolCommand>,
        nonce: &mut Nonce,
    ) -> QueryResult<usize> {
        use crate::database::schema::lend_pool_command::dsl::*;

        let items: Vec<LendPoolCommandUpdate> = updates
            .into_iter()
            .flat_map(|cmd| lend_pool_to_batch(cmd, nonce))
            .collect();

        let _ = nonce.save(conn)?;
        diesel::insert_into(lend_pool_command)
            .values(items)
            .execute(conn)
    }
}

fn lend_pool_to_batch(
    item: relayer_db::LendPoolCommand,
    pool_nonce: &mut Nonce,
) -> Vec<LendPoolCommandUpdate> {
    match item {
        relayer_db::LendPoolCommand::AddTraderOrderSettlement(_, order, p) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::ADD_TRADER_ORDER_SETTLEMENT, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::AddTraderLimitOrderSettlement(_, order, p) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(
                LendPoolCommandType::ADD_TRADER_LIMIT_ORDER_SETTLEMENT,
                uuid,
                pay,
            )
                .into()]
        }
        relayer_db::LendPoolCommand::AddFundingData(order, p) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::ADD_FUNDING_DATA, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::AddTraderOrderLiquidation(_, order, p) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::ADD_TRADER_ORDER_LIQUIDATION, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::LendOrderCreateOrder(_, order, p) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::LEND_ORDER_CREATE_ORDER, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::LendOrderSettleOrder(_, order, p) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::LEND_ORDER_SETTLE_ORDER, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::BatchExecuteTraderOrder(relayer_command) => {
            match relayer_command {
                relayer::RelayerCommand::FundingCycle(batch_order, _meta, _time) => {
                    let relayer_db::PoolBatchOrder {
                        trader_order_data, ..
                    } = batch_order;

                    trader_order_data
                        .into_iter()
                        .flat_map(|cmd| lend_pool_to_batch(cmd, pool_nonce))
                        .collect()
                }
                relayer::RelayerCommand::RpcCommandPoolupdate() => {
                    let _ = pool_nonce.increment();
                    vec![]
                }
                o => {
                    panic!("Relayer command {:?} not handled", o)
                }
            }
        }
        relayer_db::LendPoolCommand::InitiateNewPool(order, _) => {
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::INITIATE_NEW_POOL, uuid, None).into()]
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = sorted_set_command)]
pub struct SortedSetCommand {
    id: i64,
    command: SortedSetCommandType,
    uuid: Option<Uuid>,
    amount: Option<BigDecimal>,
    position_type: PositionType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = sorted_set_command)]
pub struct SortedSetCommandUpdate {
    command: SortedSetCommandType,
    uuid: Option<Uuid>,
    amount: Option<BigDecimal>,
    position_type: PositionType,
}

impl SortedSetCommand {
    pub fn append(
        conn: &mut PgConnection,
        updates: Vec<relayer::SortedSetCommand>,
    ) -> QueryResult<usize> {
        use crate::database::schema::sorted_set_command::dsl::*;

        let items: Vec<_> = updates
            .into_iter()
            .map(|item| {
                let (cmd, cmd_uuid, amt, typ) = match item {
                    relayer::SortedSetCommand::AddLiquidationPrice(i, amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::ADD_LIQUIDATION_PRICE,
                            cmd_uuid,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::AddOpenLimitPrice(i, amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::ADD_OPEN_LIMIT_PRICE,
                            cmd_uuid,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::AddCloseLimitPrice(i, amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::ADD_CLOSE_LIMIT_PRICE,
                            cmd_uuid,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::RemoveLiquidationPrice(i, typ) => {
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::REMOVE_LIQUIDATION_PRICE,
                            cmd_uuid,
                            None,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::RemoveOpenLimitPrice(i, typ) => {
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::REMOVE_OPEN_LIMIT_PRICE,
                            cmd_uuid,
                            None,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::RemoveCloseLimitPrice(i, typ) => {
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::REMOVE_CLOSE_LIMIT_PRICE,
                            cmd_uuid,
                            None,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::UpdateLiquidationPrice(i, amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::UPDATE_LIQUIDATION_PRICE,
                            cmd_uuid,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::UpdateOpenLimitPrice(i, amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::UPDATE_OPEN_LIMIT_PRICE,
                            cmd_uuid,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::UpdateCloseLimitPrice(i, amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        let cmd_uuid = Some(Uuid::from_bytes(*i.as_bytes()));
                        (
                            SortedSetCommandType::UPDATE_CLOSE_LIMIT_PRICE,
                            cmd_uuid,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::BulkSearchRemoveLiquidationPrice(amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        (
                            SortedSetCommandType::BULK_SEARCH_REMOVE_LIQUIDATION_PRICE,
                            None,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::BulkSearchRemoveOpenLimitPrice(amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        (
                            SortedSetCommandType::BULK_SEARCH_REMOVE_OPEN_LIMIT_PRICE,
                            None,
                            amt,
                            typ,
                        )
                    }
                    relayer::SortedSetCommand::BulkSearchRemoveCloseLimitPrice(amt, typ) => {
                        let amt = Some(BigDecimal::from_f64(amt).expect("Invalid f64"));
                        (
                            SortedSetCommandType::BULK_SEARCH_REMOVE_CLOSE_LIMIT_PRICE,
                            None,
                            amt,
                            typ,
                        )
                    }
                };

                SortedSetCommandUpdate {
                    command: cmd,
                    uuid: cmd_uuid,
                    amount: amt,
                    position_type: typ.into(),
                }
            })
            .collect();

        diesel::insert_into(sorted_set_command)
            .values(items)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = position_size_log)]
pub struct PositionSizeLog {
    pub id: i64,
    pub command: PositionSizeCommand,
    pub position_type: PositionType,
    pub amount: BigDecimal,
    pub total_short: BigDecimal,
    pub total_long: BigDecimal,
    pub total: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = position_size_log)]
pub struct PositionSizeLogUpdate {
    pub command: PositionSizeCommand,
    pub position_type: PositionType,
    pub amount: BigDecimal,
    pub total_short: BigDecimal,
    pub total_long: BigDecimal,
    pub total: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = position_size_log)]
pub struct PositionSize {
    pub total_short: BigDecimal,
    pub total_long: BigDecimal,
    pub total: BigDecimal,
}

impl PositionSizeLog {
    pub fn append(conn: &mut PgConnection, sizes: Vec<PositionSizeUpdate>) -> QueryResult<usize> {
        use crate::database::schema::position_size_log::dsl::*;

        let items: Vec<_> = sizes
            .into_iter()
            .map(|item| {
                let (cmd, log) = item;

                let (cmd, typ, amt) = match cmd {
                    relayer::PositionSizeLogCommand::AddPositionSize(typ, amt) => {
                        (PositionSizeCommand::ADD, typ, amt)
                    }
                    relayer::PositionSizeLogCommand::RemovePositionSize(typ, amt) => {
                        (PositionSizeCommand::REMOVE, typ, amt)
                    }
                };

                PositionSizeLogUpdate {
                    command: cmd,
                    position_type: typ.into(),
                    amount: BigDecimal::from_f64(amt).expect("Invalid f64"),
                    total_short: BigDecimal::from_f64(log.total_short_positionsize)
                        .expect("Invalid f64"),
                    total_long: BigDecimal::from_f64(log.total_long_positionsize)
                        .expect("Invalid f64"),
                    total: BigDecimal::from_f64(log.totalpositionsize).expect("Invalid f64"),
                }
            })
            .collect();

        diesel::insert_into(position_size_log)
            .values(items)
            .execute(conn)
    }

    pub fn get_latest(conn: &mut PgConnection) -> QueryResult<PositionSize> {
        use crate::database::schema::position_size_log::dsl::*;

        position_size_log
            .select((total_short, total_long, total))
            .order(id.desc())
            .first(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = btc_usd_price)]
pub struct BtcUsdPrice {
    pub id: i64,
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, QueryableByName)]
pub struct CandleData {
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub start: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub end: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub low: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub high: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub open: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub close: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub resolution: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub btc_volume: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub trades: i64,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub usd_volume: BigDecimal,
}

impl BtcUsdPrice {
    pub fn get(conn: &mut PgConnection) -> QueryResult<BtcUsdPrice> {
        use crate::database::schema::btc_usd_price::dsl::*;

        btc_usd_price.order_by(timestamp.desc()).first(conn)
    }

    pub fn get_historical(
        conn: &mut PgConnection,
        args: HistoricalPriceArgs,
    ) -> QueryResult<Vec<BtcUsdPrice>> {
        use crate::database::schema::btc_usd_price::dsl::*;
        let HistoricalPriceArgs {
            from,
            to,
            limit,
            offset,
        } = args;

        btc_usd_price
            .filter(diesel::BoolExpressionMethods::and(
                timestamp.ge(from),
                timestamp.lt(to),
            ))
            .limit(limit)
            .offset(offset)
            .load(conn)
    }

    pub fn candles(
        conn: &mut PgConnection,
        interval: Interval,
        since: DateTime<Utc>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> QueryResult<Vec<CandleData>> {
        let interval = interval.interval_sql();

        let trader_subquery = format!(
            r#"
            SELECT
                window_start,
                sum(entryprice) as usd_volume,
                sum(positionsize) as btc_volume,
                count(*) as trades
            FROM (
                SELECT
                    t.timestamp as window_start,
                    coalesce(c.entryprice, 0) as entryprice,
                    coalesce(c.positionsize, 0) as positionsize
                FROM generate_series('{}', now(), {}) t(timestamp)
                LEFT JOIN trader_order c
                ON c.timestamp BETWEEN t.timestamp AND t.timestamp + {}
            ) as sq
            GROUP BY window_start
        "#,
            since, interval, interval
        );

        let ohlc_subquery = format!(
            r#"
            SELECT
                window_ts,
                min(timestamp) as start,
                max(timestamp) as end,
                min(open) as open,
                min(close) as close,
                max(price) as high,
                min(price) as low
            FROM (
                SELECT
                     t.timestamp as window_ts,
                     c.*,
                     first_value(price) OVER (PARTITION BY t.timestamp ORDER BY c.timestamp asc) AS open,
                     first_value(price) OVER (PARTITION BY t.timestamp ORDER BY c.timestamp desc) AS close
                FROM generate_series('{}', now(), {}) t(timestamp)
                LEFT JOIN btc_usd_price c
                ON c.timestamp BETWEEN t.timestamp AND t.timestamp + {} 
            ) as w
            WHERE open IS NOT NULL
            GROUP BY window_ts
        "#,
            since, interval, interval
        );

        let query = format!(
            r#"
                WITH volumes AS (
                    {}
                ), ohlc AS (
                    {}
                )
        SELECT
            ohlc.start,
            ohlc.end,
            ohlc.open,
            ohlc.close,
            ohlc.high,
            ohlc.low,
            {} as resolution,
            volumes.btc_volume as btc_volume,
            volumes.trades as trades,
            volumes.usd_volume as usd_volume
        FROM volumes
         JOIN ohlc ON volumes.window_start = ohlc.window_ts
        "#,
            trader_subquery, ohlc_subquery, interval
        );

        let query = if let Some(limit) = limit {
            format!("{} LIMIT {}", query, limit)
        } else {
            query
        };

        let query = if let Some(offset) = offset {
            format!("{} OFFSET {}", query, offset)
        } else {
            query
        };

        diesel::sql_query(query).get_results(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = btc_usd_price)]
pub struct CurrentPriceUpdate {
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

impl CurrentPriceUpdate {
    pub fn insert(
        conn: &mut PgConnection,
        current_price: f64,
        ts: DateTime<Utc>,
    ) -> QueryResult<usize> {
        use crate::database::schema::btc_usd_price::dsl::*;

        let update = CurrentPriceUpdate {
            price: BigDecimal::from_f64(current_price).unwrap(),
            timestamp: ts,
        };

        diesel::insert_into(btc_usd_price)
            .values(update)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = funding_rate)]
pub struct FundingRate {
    pub id: i64,
    pub rate: BigDecimal,
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

impl FundingRate {
    pub fn get(conn: &mut PgConnection) -> QueryResult<FundingRate> {
        use crate::database::schema::funding_rate::dsl::*;

        funding_rate.order_by(timestamp.desc()).first(conn)
    }

    pub fn get_historical(
        conn: &mut PgConnection,
        args: HistoricalFundingArgs,
    ) -> QueryResult<Vec<FundingRate>> {
        use crate::database::schema::funding_rate::dsl::*;
        let HistoricalFundingArgs {
            from,
            to,
            limit,
            offset,
        } = args;

        funding_rate
            .filter(diesel::BoolExpressionMethods::and(
                timestamp.ge(from),
                timestamp.lt(to),
            ))
            .limit(limit)
            .offset(offset)
            .load(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = funding_rate)]
pub struct FundingRateUpdate {
    pub rate: BigDecimal,
    pub price: BigDecimal,
    pub timestamp: DateTime<Utc>,
}

impl FundingRateUpdate {
    pub fn insert(
        conn: &mut PgConnection,
        r: f64,
        p: f64,
        ts: DateTime<Utc>,
    ) -> QueryResult<usize> {
        use crate::database::schema::funding_rate::dsl::*;

        let update = FundingRateUpdate {
            rate: BigDecimal::from_f64(r).unwrap(),
            price: BigDecimal::from_f64(p).unwrap(),
            timestamp: ts,
        };

        diesel::insert_into(funding_rate)
            .values(update)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = trader_order)]
pub struct TraderOrder {
    pub id: i64,
    pub uuid: String,
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

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = trader_order)]
pub struct InsertTraderOrder {
    pub uuid: String,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnrealizedPnl {
    order_ids: Vec<String>,
    pnl: f64,
}

impl TraderOrder {
    pub fn get(conn: &mut PgConnection, id: String) -> QueryResult<TraderOrder> {
        use crate::database::schema::trader_order::dsl::*;

        trader_order.find(id).first(conn)
    }

    pub fn insert(conn: &mut PgConnection, orders: Vec<InsertTraderOrder>) -> QueryResult<usize> {
        use crate::database::schema::trader_order::dsl::*;

        let query = diesel::insert_into(trader_order).values(&orders);

        query.execute(conn)
    }

    pub fn unrealized_pnl(
        conn: &mut PgConnection,
        pnl_args: PnlArgs,
    ) -> QueryResult<UnrealizedPnl> {
        use crate::database::schema::trader_order::dsl::*;

        let price = BtcUsdPrice::get(conn)?;
        let closed = vec![
            OrderStatus::FILLED,
            OrderStatus::CANCELLED,
            OrderStatus::LIQUIDATE,
            OrderStatus::SETTLED,
        ];

        let orders: Vec<TraderOrder> = match pnl_args {
            PnlArgs::OrderId(oid) => {
                // TODO: uuid broken...
                //let order = trader_order
                //    .filter(
                //        id.eq(oid).and(order_status.ne_all(closed))
                //    ).order_by(timestamp.desc()).first(conn)?;
                //vec![order]
                vec![]
            }
            PnlArgs::PublicKey(key) => trader_order
                .filter(account_id.eq(key).and(order_status.ne_all(closed)))
                .load(conn)?,
            PnlArgs::All => trader_order
                .filter(order_status.ne_all(closed))
                .load(conn)?,
        };

        let mut pnl = BigDecimal::zero();
        let order_ids = orders
            .into_iter()
            .map(|o| {
                pnl += o.unrealized_pnl;
                o.uuid.to_string()
            })
            .collect();

        Ok(UnrealizedPnl {
            order_ids,
            pnl: pnl.to_f64().unwrap(),
        })
    }

    pub fn order_history(
        conn: &mut PgConnection,
        args: OrderHistoryArgs,
    ) -> QueryResult<Vec<TraderOrder>> {
        use crate::database::schema::trader_order::dsl::*;

        match args {
            OrderHistoryArgs::OrderId(order_id) => {
                //TODO: by id??
                Ok(vec![])
            }
            OrderHistoryArgs::ClientId {
                client_id,
                offset,
                limit,
            } => {
                trader_order
                    .filter(account_id.eq(client_id))
                    //TODO: uuid...
                    //.group_by(uuid)
                    .limit(limit)
                    .offset(offset)
                    .order_by(timestamp.desc())
                    .load(conn)
            }
        }
    }

    pub fn order_volume(conn: &mut PgConnection, args: TradeVolumeArgs) -> QueryResult<usize> {
        use crate::database::schema::trader_order::dsl::*;
        trader_order
            .count()
            .filter(timestamp.between(args.start, args.end))
            .distinct_on(uuid)
            .execute(conn)
    }

    pub fn open_orders(conn: &mut PgConnection) -> QueryResult<Vec<TraderOrder>> {
        use crate::database::schema::trader_order::dsl::*;

        // TODO: filter by user-id??
        let closed = vec![
            OrderStatus::FILLED,
            OrderStatus::CANCELLED,
            OrderStatus::LIQUIDATE,
            OrderStatus::SETTLED,
        ];

        trader_order.filter(order_status.ne_all(closed)).load(conn)
    }

    pub fn list_open_limit_orders(conn: &mut PgConnection) -> QueryResult<OrderBook> {
        use crate::database::schema::trader_order::dsl::*;

        let orders = trader_order
            .filter(
                order_type
                    .eq(OrderType::LIMIT)
                    .and(order_status.eq(OrderStatus::PENDING)),
            )
            .load(conn)?;

        let mut ask = Vec::new();
        let mut bid = Vec::new();
        for order in orders {
            let TraderOrder {
                positionsize: ps,
                position_type: pt,
                entryprice: ep,
                ..
            } = order;
            if pt == PositionType::LONG {
                bid.push(Bid {
                    positionsize: ps.to_f64().unwrap(),
                    price: ep.to_f64().unwrap(),
                });
            } else {
                ask.push(Ask {
                    positionsize: ps.to_f64().unwrap(),
                    price: ep.to_f64().unwrap(),
                });
            }
        }

        let ob = OrderBook { bid, ask };

        Ok(ob)
    }

    pub fn list_past_24hrs(conn: &mut PgConnection) -> QueryResult<Vec<TraderOrder>> {
        use crate::database::schema::trader_order::dsl::*;

        let start = Utc::now() - chrono::Duration::days(1);
        trader_order
            .filter(
                timestamp
                    .ge(start)
                    .and(order_status.ne(OrderStatus::PENDING))
                    .and(order_type.ne(OrderType::LIMIT)),
            )
            .load(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderBook {
    pub bid: Vec<Bid>,
    pub ask: Vec<Ask>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ask {
    pub positionsize: f64,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bid {
    pub positionsize: f64,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = lend_order)]
pub struct LendOrder {
    pub id: i64,
    pub uuid: String,
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

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = lend_order)]
pub struct InsertLendOrder {
    pub uuid: String,
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
    pub fn get(conn: &mut PgConnection, id: String) -> QueryResult<LendOrder> {
        use crate::database::schema::lend_order::dsl::*;

        lend_order.find(id).first(conn)
    }

    pub fn insert(conn: &mut PgConnection, orders: Vec<InsertLendOrder>) -> QueryResult<usize> {
        use crate::database::schema::lend_order::dsl::*;

        let query = diesel::insert_into(lend_order).values(&orders);

        query.execute(conn)
    }
}

impl From<relayer::TraderOrder> for InsertTraderOrder {
    fn from(src: relayer::TraderOrder) -> InsertTraderOrder {
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

        InsertTraderOrder {
            uuid: uuid.to_string(),
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
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
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

impl From<relayer::LendOrder> for InsertLendOrder {
    fn from(src: relayer::LendOrder) -> InsertLendOrder {
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

        InsertLendOrder {
            uuid: uuid.to_string(),
            account_id,
            balance: BigDecimal::from_f64(balance).unwrap(),
            order_status: order_status.into(),
            order_type: order_type.into(),
            entry_nonce: entry_nonce as i64,
            exit_nonce: exit_nonce as i64,
            deposit: BigDecimal::from_f64(deposit).unwrap(),
            new_lend_state_amount: BigDecimal::from_f64(new_lend_state_amount).unwrap(),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
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
            uuid: bytes.encode_hex::<String>(),
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
            uuid: bytes.encode_hex::<String>(),
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
