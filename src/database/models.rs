#![allow(warnings)]
use crate::database::{
    schema::{
        address_customer_id, btc_usd_price, current_nonce, customer_account,
        customer_apikey_linking, customer_order_linking, funding_rate, lend_order, lend_pool,
        lend_pool_command, position_size_log, sorted_set_command, trader_order,
        trader_order_funding_updated, transaction_hash,
    },
    sql_types::*,
};
use crate::rpc::{
    HistoricalFundingArgs, HistoricalPriceArgs, Interval, OrderHistoryArgs, OrderId, PnlArgs,
    TradeVolumeArgs, TransactionHashArgs,
};
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero};
use chrono::{prelude::*, DurationRound};
// use diesel::pg::Pg;
use diesel::prelude::*;
use itertools::join;
use serde::{Deserialize, Serialize};
use twilight_relayer_rust::{db as relayer_db, relayer};
use uuid::Uuid;

pub type PositionSizeUpdate = (relayer::PositionSizeLogCommand, relayer_db::PositionSizeLog);

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, QueryableByName)]
#[diesel(table_name = transaction_hash)]
pub struct TxHash {
    pub id: i64,
    pub order_id: String,
    pub account_id: String,
    pub tx_hash: String,
    pub order_type: OrderType,
    pub order_status: OrderStatus,
    pub datetime: String,
    pub output: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = transaction_hash)]
pub struct NewTxHash {
    pub order_id: String,
    pub account_id: String,
    pub tx_hash: String,
    pub order_type: OrderType,
    pub order_status: OrderStatus,
    pub datetime: String,
    pub output: Option<String>,
    pub request_id: Option<String>,
}

impl TxHash {
    pub fn get(conn: &mut PgConnection, args: TransactionHashArgs) -> QueryResult<Vec<TxHash>> {
        use crate::database::schema::transaction_hash::dsl::*;

        match args {
            TransactionHashArgs::TxId { id: tx_id, status } => {
                if let Some(status) = status {
                    transaction_hash
                        .filter(order_id.eq(tx_id).and(order_status.eq(status)))
                        .load(conn)
                } else {
                    transaction_hash.filter(order_id.eq(tx_id)).load(conn)
                }
            }
            TransactionHashArgs::AccountId {
                id: acct_id,
                status,
            } => {
                if let Some(status) = status {
                    transaction_hash
                        .filter(account_id.eq(acct_id).and(order_status.eq(status)))
                        .load(conn)
                } else {
                    transaction_hash.filter(account_id.eq(acct_id)).load(conn)
                }
            }
            TransactionHashArgs::RequestId {
                id: reqt_id,
                status,
            } => {
                if let Some(status) = status {
                    transaction_hash
                        .filter(request_id.eq(reqt_id).and(order_status.eq(status)))
                        .load(conn)
                } else {
                    transaction_hash.filter(request_id.eq(reqt_id)).load(conn)
                }
            }
        }
    }

    pub fn create(conn: &mut PgConnection, new: NewTxHash) -> QueryResult<()> {
        use crate::database::schema::transaction_hash::dsl::*;

        diesel::insert_into(transaction_hash)
            .values(new)
            .execute(conn)?;

        Ok(())
    }

    pub fn append(conn: &mut PgConnection, new_hashes: Vec<NewTxHash>) -> QueryResult<usize> {
        use crate::database::schema::transaction_hash::dsl::*;

        diesel::insert_into(transaction_hash)
            .values(new_hashes)
            .execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_account)]
pub struct CustomerAccount {
    pub id: i64,
    pub customer_registration_id: String,
    pub username: String,
    pub password: String,
    pub created_on: DateTime<Utc>,
    pub password_hint: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_account)]
pub struct NewCustomerAccount {
    pub customer_registration_id: String,
    pub username: String,
    pub password: String,
    pub created_on: DateTime<Utc>,
    pub password_hint: String,
}

impl CustomerAccount {
    pub fn get(conn: &mut PgConnection, ident: i64) -> QueryResult<CustomerAccount> {
        use crate::database::schema::customer_account::dsl::*;

        customer_account.find(ident).first(conn)
    }

    pub fn create(
        conn: &mut PgConnection,
        new_account: NewCustomerAccount,
    ) -> QueryResult<Vec<CustomerAccount>> {
        use crate::database::schema::customer_account::dsl::*;

        diesel::insert_into(customer_account)
            .values(new_account)
            .get_results(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = address_customer_id)]
pub struct AddressCustomerId {
    pub id: i64,
    pub address: String,
    pub customer_id: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = address_customer_id)]
pub struct NewAddressCustomerId {
    pub address: String,
    pub customer_id: i64,
}

impl AddressCustomerId {
    pub fn get(conn: &mut PgConnection, addr: &String) -> QueryResult<Option<AddressCustomerId>> {
        use crate::database::schema::address_customer_id::dsl::*;

        match address_customer_id.filter(address.eq(addr)).first(conn) {
            Ok(o) => return Ok(Some(o)),
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e),
        }
    }

    pub fn insert(conn: &mut PgConnection, customer: i64, addr: &String) -> QueryResult<()> {
        use crate::database::schema::address_customer_id::dsl::*;

        let address_id = NewAddressCustomerId {
            customer_id: customer,
            address: addr.to_string(),
        };

        diesel::insert_into(address_customer_id)
            .values(address_id)
            .on_conflict_do_nothing()
            .execute(conn)?;

        Ok(())
    }

    pub fn create(
        conn: &mut PgConnection,
        addr: &String,
    ) -> QueryResult<Option<AddressCustomerId>> {
        use crate::database::schema::address_customer_id::dsl::*;

        match address_customer_id
            .filter(address.eq(addr))
            .first::<AddressCustomerId>(conn)
        {
            Ok(_o) => return Ok(None),
            Err(diesel::result::Error::NotFound) => {
                let mut account = NewCustomerAccount::default();
                account.customer_registration_id = Uuid::new_v4().to_string();

                let customer = CustomerAccount::create(conn, account)?;
                let address_id = NewAddressCustomerId {
                    customer_id: customer[0].id,
                    address: addr.to_string(),
                };

                let account: Vec<AddressCustomerId> = diesel::insert_into(address_customer_id)
                    .values(address_id)
                    .get_results(conn)?;

                return Ok(Some(account[0].clone()));
            }
            Err(e) => return Err(e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_apikey_linking)]
pub struct CustomerApiKeyLinking {
    pub id: i64,
    pub customer_account_id: i64,
    pub api_key: String,
    pub api_salt_key: String,
    pub created_on: DateTime<Utc>,
    pub expires_on: DateTime<Utc>,
    pub is_active: bool,
    pub remark: Option<String>,
    pub authorities: Option<String>,
    pub limit_remaining: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, Queryable)]
#[diesel(table_name = customer_apikey_linking)]
pub struct NewCustomerApiKeyLinking {
    pub customer_account_id: i64,
    pub api_key: String,
    pub api_salt_key: String,
    pub created_on: DateTime<Utc>,
    pub expires_on: DateTime<Utc>,
    pub is_active: bool,
    pub remark: Option<String>,
    pub authorities: Option<String>,
    pub limit_remaining: Option<i64>,
}

impl CustomerApiKeyLinking {
    pub fn get_key(conn: &mut PgConnection, key: String) -> QueryResult<CustomerApiKeyLinking> {
        use crate::database::schema::customer_apikey_linking::dsl::*;

        customer_apikey_linking
            .filter(api_key.eq(key).and(is_active.eq(true)))
            .first(conn)
    }

    pub fn regenerate(
        conn: &mut PgConnection,
        customer_id: i64,
    ) -> QueryResult<CustomerApiKeyLinking> {
        use crate::database::schema::customer_apikey_linking::dsl::*;

        diesel::update(customer_apikey_linking)
            .filter(customer_account_id.eq(customer_id))
            .set(is_active.eq(false))
            .execute(conn)?;

        Self::create(conn, customer_id)
    }

    pub fn create(conn: &mut PgConnection, customer_id: i64) -> QueryResult<CustomerApiKeyLinking> {
        use crate::database::schema::customer_apikey_linking::dsl::*;

        let linking = NewCustomerApiKeyLinking {
            api_key: Uuid::new_v4().to_string(),
            api_salt_key: Uuid::new_v4().to_string(),
            customer_account_id: customer_id,
            created_on: Utc::now(),
            expires_on: Utc::now() + chrono::Duration::days(7),
            is_active: true,
            remark: None,
            authorities: None,
            limit_remaining: None,
        };

        let result: Vec<CustomerApiKeyLinking> = diesel::insert_into(customer_apikey_linking)
            .values(linking)
            .get_results(conn)?;

        return Ok(result[0].clone());
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

        lend_pool.order_by(nonce.desc()).first(conn)
    }
    pub fn get_pool_share_value(&self) -> f64 {
        let tps = self.total_pool_share.to_f64().unwrap_or(1.0);
        let tlv = self.total_locked_value.to_f64().unwrap_or(0.0);
        tlv / tps * 100.0
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
                total_pool_share: BigDecimal::from_f64(update.total_pool_share)
                    .expect("Invalid floating point"),
                total_locked_value: BigDecimal::from_f64(update.total_locked_value)
                    .expect("Invalid floating point"),
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
        relayer_db::LendPoolCommand::AddTraderOrderSettlement(_, order, p, _) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::ADD_TRADER_ORDER_SETTLEMENT, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::AddTraderLimitOrderSettlement(_, order, p, _) => {
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
        relayer_db::LendPoolCommand::AddTraderOrderLiquidation(_, order, p, _) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::ADD_TRADER_ORDER_LIQUIDATION, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::LendOrderCreateOrder(_, order, p, _) => {
            let pay = Some(BigDecimal::from_f64(p).expect("Invalid floating point number"));
            let uuid = order.uuid.to_string();
            vec![(LendPoolCommandType::LEND_ORDER_CREATE_ORDER, uuid, pay).into()]
        }
        relayer_db::LendPoolCommand::LendOrderSettleOrder(_, order, p, _) => {
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
        relayer_db::LendPoolCommand::InitiateNewPool(order, _, payment) => {
            let uuid = order.uuid.to_string();
            let pay = Some(BigDecimal::from_f64(payment).expect("Invalid flaoting point"));
            vec![(LendPoolCommandType::INITIATE_NEW_POOL, uuid, pay).into()]
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable)]
#[diesel(table_name = sorted_set_command)]
pub struct SortedSetCommand {
    id: i64,
    command: SortedSetCommandType,
    uuid: Option<String>,
    amount: Option<BigDecimal>,
    position_type: PositionType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = sorted_set_command)]
pub struct SortedSetCommandUpdate {
    command: SortedSetCommandType,
    uuid: Option<String>,
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
                    uuid: cmd_uuid.map(|m| m.to_string()),
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Queryable, QueryableByName)]
pub struct CandleData {
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
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
    pub fn update_candles(conn: &mut PgConnection) -> QueryResult<()> {
        diesel::sql_query("SELECT * from  update_candles_1min()").execute(conn)?;
        diesel::sql_query("SELECT * from  update_candles_1hour()").execute(conn)?;
        diesel::sql_query("SELECT * from  update_candles_1day()").execute(conn)?;

        Ok(())
    }

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
        let start: DateTime<Utc>;
        let table: String;
        // temp for 24 hour candle change
        // need to create new api for 24hour candle change data
        match interval {
            Interval::ONE_DAY_CHANGE => {
                start = since + chrono::Duration::seconds(5);
                table = "candles_1hour".into()
            }
            Interval::ONE_MINUTE
            | Interval::FIVE_MINUTE
            | Interval::FIFTEEN_MINUTE
            | Interval::THIRTY_MINUTE => {
                start = since.duration_trunc(interval.duration()).unwrap();
                table = "candles_1min".into()
            }
            Interval::ONE_HOUR
            | Interval::FOUR_HOUR
            | Interval::EIGHT_HOUR
            | Interval::TWELVE_HOUR => {
                start = since.duration_trunc(interval.duration()).unwrap();
                table = "candles_1hour".into()
            }
            Interval::ONE_DAY => {
                start = since.duration_trunc(interval.duration()).unwrap();
                table = "candles_1day".into()
            }
        }
        let interval = interval.interval_sql();

        let subquery = format!(
            r#"
            with t as (
                select * from
                generate_series('{}', now(), {}) timestamp
            ), c as (
                select * from {}
                where start_time between '{}' and now()
            )
            select
               t.timestamp as bucket,
               c.start_time as start,
               c.open,
               c.close,
               c.high,
               c.low,
               c.btc_volume,
               c.trades,
               c.usd_volume
            from t
            inner join c
            on c.start_time >= t.timestamp AND c.start_time < t.timestamp + interval {} order by c.start_time
            "#,
            start, interval, table, start, interval
        );

        let query = format!(
            r#"
        select distinct
            now() as updated_at,
            {} as resolution,
            bucket as start,
            bucket + interval {} as end,
            max(high) over w as high,
            min(low) over w as low,
            last_value(close) over w as close,
            first_value(open) over w as open,
            sum(btc_volume) over w as btc_volume,
            sum(usd_volume) over w as usd_volume,
            sum(trades) over w as trades
        from (
            {}
        ) as s
        WINDOW w as (partition by bucket order by start asc ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
        order by start asc
        "#,
            interval, interval, subquery,
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
            price: BigDecimal::from_f64(current_price).unwrap().round(2),
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

#[derive(Serialize, Deserialize, Default, Debug, Clone, QueryableByName, Queryable)]
pub struct Payment {
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub funding_payment: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundingPayment {
    pub order_id: String,
    pub funding_rate: BigDecimal,
    pub price: BigDecimal,
    pub funding_payment: BigDecimal,
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

    pub fn funding_payment(
        conn: &mut PgConnection,
        customer_id: i64,
        order_id: String,
    ) -> QueryResult<FundingPayment> {
        use crate::database::schema::address_customer_id::dsl as acct_dsl;
        use crate::database::schema::funding_rate::dsl::*;

        let accounts: Vec<AddressCustomerId> = acct_dsl::address_customer_id
            .filter(acct_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let iter = accounts.into_iter().map(|a| format!("'{}'", a.address));
        let accounts = join(iter, ", ");

        let fr: FundingRate = funding_rate.order_by(timestamp.desc()).first(conn)?;

        let query = format!(
            r#"
        SELECT coalesce(payment, 0) as funding_payment
        FROM (SELECT LEAD(available_margin)
            OVER (ORDER BY timestamp DESC) - available_margin AS payment 
            FROM trader_order
            WHERE uuid = '{}'
            AND account_id IN ({})
        ) t"#,
            order_id, accounts
        );

        let resp: Vec<Payment> = diesel::sql_query(query).load(conn)?;
        let pmnt = resp.get(0).cloned().unwrap_or_default();

        Ok(FundingPayment {
            order_id,
            funding_rate: fr.rate,
            price: fr.price,
            funding_payment: pmnt.funding_payment,
        })
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
            rate: BigDecimal::from_f64(r).unwrap().round(6),
            price: BigDecimal::from_f64(p).unwrap().round(2),
            timestamp: ts,
        };

        diesel::insert_into(funding_rate)
            .values(update)
            .execute(conn)
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, QueryableByName, Queryable, Insertable, AsChangeset,
)]
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

#[derive(
    Serialize, Deserialize, Debug, Clone, QueryableByName, Queryable, Insertable, AsChangeset,
)]
#[diesel(table_name = trader_order_funding_updated)]
pub struct TraderOrderFundingUpdates {
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

#[derive(Serialize, Deserialize, Debug, Clone, QueryableByName, Queryable)]
pub struct RecentOrder {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub order_id: String,
    #[diesel(sql_type = crate::database::schema::sql_types::PositionType)]
    pub side: PositionType,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub positionsize: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, QueryableByName, Queryable)]
pub struct TradeVolume {
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub volume: BigDecimal,
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

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = trader_order_funding_updated)]
pub struct InsertTraderOrderFundingUpdates {
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

#[derive(
    Serialize, Deserialize, Debug, Clone, Queryable, QueryableByName, Insertable, AsChangeset,
)]
#[diesel(table_name = trader_order)]
pub struct OrderBookOrder {
    uuid: String,
    entryprice: BigDecimal,
    positionsize: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NewOrderBookOrder {
    Bid {
        id: String,
        positionsize: f64,
        price: f64,
    },
    Ask {
        id: String,
        positionsize: f64,
        price: f64,
    },
}

impl NewOrderBookOrder {
    pub fn new(to: TraderOrder) -> Self {
        if to.position_type == PositionType::LONG {
            Self::Bid {
                id: to.uuid,
                positionsize: to.positionsize.to_f64().unwrap(),
                price: to.entryprice.to_f64().unwrap(),
            }
        } else {
            Self::Ask {
                id: to.uuid,
                positionsize: to.positionsize.to_f64().unwrap(),
                price: to.entryprice.to_f64().unwrap(),
            }
        }
    }
}

pub fn unrealizedpnl(
    position_type: &PositionType,
    positionsize: &BigDecimal,
    entryprice: &BigDecimal,
    settleprice: &BigDecimal,
) -> BigDecimal {
    if entryprice > &BigDecimal::zero() && settleprice > &BigDecimal::zero() {
        match position_type {
            &PositionType::LONG => {
                (positionsize * (settleprice - entryprice)) / (entryprice * settleprice)
            }
            &PositionType::SHORT => {
                (positionsize * (entryprice - settleprice)) / (entryprice * settleprice)
            }
        }
    } else {
        BigDecimal::zero()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnrealizedPnl {
    order_ids: Vec<String>,
    pnl: f64,
}

impl TraderOrder {
    pub fn get(
        conn: &mut PgConnection,
        customer_id: i64,
        order_id: String,
    ) -> QueryResult<TraderOrder> {
        use crate::database::schema::address_customer_id::dsl as addr_dsl;
        use crate::database::schema::trader_order::dsl::*;

        let accounts: Vec<AddressCustomerId> = addr_dsl::address_customer_id
            .filter(addr_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let accounts: Vec<_> = accounts.into_iter().map(|a| a.address).collect();

        trader_order
            .filter(uuid.eq(order_id).and(account_id.eq_any(accounts)))
            .order(timestamp.desc())
            .first(conn)
    }
    pub fn get_by_signature(
        conn: &mut PgConnection,
        accountid: String,
    ) -> QueryResult<TraderOrder> {
        use crate::database::schema::trader_order::dsl::*;
        trader_order
            .filter(account_id.eq(accountid))
            .order(timestamp.desc())
            .first(conn)
    }
    pub fn get_by_uuid(conn: &mut PgConnection, order_id: String) -> QueryResult<TraderOrder> {
        // use crate::database::schema::address_customer_id::dsl as addr_dsl;
        use crate::database::schema::trader_order::dsl::*;

        trader_order
            .filter(uuid.eq(order_id))
            .order(timestamp.desc())
            .first(conn)
    }

    pub fn insert(conn: &mut PgConnection, orders: Vec<InsertTraderOrder>) -> QueryResult<usize> {
        use crate::database::schema::trader_order::dsl::*;

        let query = diesel::insert_into(trader_order).values(&orders);

        query.execute(conn)
    }

    pub fn unrealized_pnl(
        conn: &mut PgConnection,
        customer_id: i64,
        pnl_args: PnlArgs,
    ) -> QueryResult<UnrealizedPnl> {
        use crate::database::schema::address_customer_id::dsl as addr_dsl;
        use crate::database::schema::trader_order::dsl::*;

        let accounts: Vec<AddressCustomerId> = addr_dsl::address_customer_id
            .filter(addr_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let accounts: Vec<_> = accounts.into_iter().map(|a| a.address).collect();

        let _price = BtcUsdPrice::get(conn)?;
        let closed = vec![
            OrderStatus::PENDING,
            OrderStatus::CANCELLED,
            OrderStatus::LIQUIDATE,
            OrderStatus::SETTLED,
        ];

        let orders: Vec<TraderOrder> = match pnl_args {
            PnlArgs::OrderId(oid) => {
                let order = trader_order
                    .filter(
                        uuid.eq(oid)
                            .and(order_status.ne_all(closed))
                            .and(account_id.eq_any(accounts)),
                    )
                    .order_by(timestamp.desc())
                    .first(conn)?;
                vec![order]
            }
            PnlArgs::PublicKey(key) => {
                let index = accounts.iter().position(|a| a == &key);

                let iter = accounts.into_iter().map(|a| format!("'{}'", a));
                let accounts = join(iter, ", ");

                if index.is_some() {
                    let query = format!(
                        r#"SELECT DISTINCT ON (uuid)
                        * FROM trader_order
                        WHERE account_id IN ({})
                        AND order_status NOT IN ('PENDING', 'CANCELLED', 'LIQUIDATE', 'SETTLED')
                        ORDER BY uuid, timestamp DESC"#,
                        accounts,
                    );
                    diesel::sql_query(query).load(conn)?
                } else {
                    vec![]
                }
            }
            PnlArgs::All => {
                let iter = accounts.into_iter().map(|a| format!("'{}'", a));
                let accounts = join(iter, ", ");

                let query = format!(
                    r#"SELECT DISTINCT ON (uuid)
                    * FROM trader_order
                    WHERE account_id IN ({})
                    AND order_status NOT IN ('PENDING', 'CANCELLED', 'LIQUIDATE', 'SETTLED')
                    ORDER BY uuid, timestamp DESC"#,
                    accounts,
                );
                diesel::sql_query(query).load(conn)?
            }
        };

        let current = BtcUsdPrice::get(conn)?;

        let mut pnl = BigDecimal::zero();
        let order_ids = orders
            .into_iter()
            .map(|o| {
                let t = unrealizedpnl(
                    &o.position_type,
                    &o.positionsize,
                    &o.entryprice,
                    &current.price,
                );
                pnl += t;
                o.uuid.to_string()
            })
            .collect();

        Ok(UnrealizedPnl {
            order_ids,
            pnl: pnl.to_f64().unwrap(),
        })
    }

    pub fn last_order(conn: &mut PgConnection, customer_id: i64) -> QueryResult<TraderOrder> {
        use crate::database::schema::address_customer_id::dsl as acct_dsl;
        use crate::database::schema::trader_order::dsl::*;

        let accounts: Vec<AddressCustomerId> = acct_dsl::address_customer_id
            .filter(acct_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let accounts: Vec<_> = accounts.into_iter().map(|a| a.address).collect();

        trader_order
            .filter(account_id.eq_any(accounts))
            .order_by(timestamp.desc())
            .first(conn)
    }

    pub fn order_history(
        conn: &mut PgConnection,
        customer_id: i64,
        args: OrderHistoryArgs,
    ) -> QueryResult<Vec<TraderOrder>> {
        use crate::database::schema::address_customer_id::dsl as acct_dsl;
        use crate::database::schema::trader_order::dsl::*;

        let accounts: Vec<AddressCustomerId> = acct_dsl::address_customer_id
            .filter(acct_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let accounts: Vec<_> = accounts.into_iter().map(|a| a.address).collect();

        match args {
            OrderHistoryArgs::OrderId(order_id) => trader_order
                .filter(account_id.eq_any(accounts).and(uuid.eq(order_id)))
                .load(conn),
            OrderHistoryArgs::ClientId {
                from,
                to,
                offset,
                limit,
            } => trader_order
                .filter(account_id.eq_any(accounts).and(timestamp.between(from, to)))
                .limit(limit)
                .offset(offset)
                .order_by(timestamp.desc())
                .load(conn),
        }
    }

    pub fn order_volume(
        conn: &mut PgConnection,
        customer_id: i64,
        args: TradeVolumeArgs,
    ) -> QueryResult<f64> {
        use crate::database::schema::address_customer_id::dsl as acct_dsl;
        // use crate::database::schema::trader_order::dsl::*;

        let accounts: Vec<AddressCustomerId> = acct_dsl::address_customer_id
            .filter(acct_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let iter = accounts.into_iter().map(|a| format!("'{}'", a.address));
        let accounts = join(iter, ", ");

        let query = format!(
            r#"SELECT coalesce(sum(positionsize), 0) as volume FROM (
            SELECT uuid, timestamp, account_id, positionsize, row_number()
            OVER (PARTITION BY uuid ORDER BY timestamp DESC) AS row_number
            FROM trader_order
        ) t where row_number = 1
        AND account_id IN ({})
        AND timestamp BETWEEN '{}' and '{}'"#,
            accounts, args.start, args.end
        );

        let size: Vec<TradeVolume> = diesel::sql_query(query).load(conn)?;

        let tv: TradeVolume = size.get(0).cloned().unwrap_or_default();
        Ok(tv.volume.to_f64().unwrap())
    }

    pub fn order_book_orders(conn: &mut PgConnection) -> QueryResult<Vec<TraderOrder>> {
        let query = r#"
            SELECT * FROM trader_order
            WHERE id IN (
                SELECT MAX(id) FROM trader_order
                WHERE order_type = 'LIMIT'
                GROUP BY uuid
            )
            AND order_status NOT IN ('FILLED', 'CANCELLED', 'LIQUIDATE')
        "#;

        diesel::sql_query(query).get_results(conn)
    }

    pub fn order_book(conn: &mut PgConnection) -> QueryResult<OrderBook> {
        // use crate::database::schema::trader_order::dsl::*;
        // use diesel::dsl::{max, sum};

        let query = r#"
        WITH orders AS (
            SELECT * FROM trader_order
            WHERE id IN (
                SELECT MAX(id) FROM trader_order 
                WHERE order_type = 'LIMIT' AND position_type = 'SHORT'
                GROUP BY uuid
            )
            AND order_status <> 'FILLED'  AND order_status <> 'CANCELLED'  AND order_status <> 'LIQUIDATE'
        ), commands AS (
            SELECT MAX(id) as id,uuid FROM sorted_set_command
            WHERE uuid IN ( SELECT uuid FROM orders )
            GROUP BY uuid
        ), updates AS (
            SELECT * FROM sorted_set_command
            WHERE id IN ( SELECT id FROM commands )
        ), updated AS(
            SELECT
                orders.uuid as uuid,
                COALESCE(amount, entryprice) as entryprice,
                positionsize as positionsize,
                updates.command as command
            FROM orders
            LEFT JOIN updates
            ON updates.uuid = orders.uuid
        ), sorted_set AS (
            SELECT *
            FROM sorted_set_command
            WHERE id IN (
                SELECT MAX(id) FROM sorted_set_command
                WHERE uuid IS NOT NULL
                GROUP BY uuid
            )
            AND command IN ('ADD_CLOSE_LIMIT_PRICE', 'UPDATE_CLOSE_LIMIT_PRICE')
            AND position_type ='SHORT' ORDER BY id DESC
        )
        SELECT
            trader_o.uuid AS uuid,
            sort.amount AS entryprice,
            trader_o.positionsize as positionsize
        FROM (
            SELECT *
            FROM trader_order
            WHERE id IN (
                SELECT MAX(id) FROM trader_order
                WHERE position_type = 'LONG' AND uuid IN (
                    SELECT uuid
                    FROM sorted_Set
                )
                GROUP BY uuid
            )
            AND order_status = 'FILLED'
        ) AS trader_o
        LEFT OUTER JOIN (
            SELECT amount,uuid FROM sorted_Set
        ) AS sort
        ON trader_o.uuid = sort.uuid
        UNION ALL
        SELECT
            MAX(uuid) AS uuid,
            entryprice,
            SUM(positionsize) AS positionsize
        FROM updated
        WHERE command IS NULL OR command <> 'REMOVE_CLOSE_LIMIT_PRICE'
        GROUP BY entryprice
        ORDER BY entryprice ASC
        LIMIT 15;
        "#;

        let shorts: Vec<OrderBookOrder> = diesel::sql_query(query).get_results(conn)?;

        let ask: Vec<_> = shorts
            .into_iter()
            .map(|order| Ask {
                id: order.uuid,
                positionsize: order.positionsize.to_f64().unwrap(),
                price: order.entryprice.to_f64().unwrap(),
            })
            .collect();

        let query = r#"
            WITH orders AS (
                SELECT * FROM trader_order
                WHERE id IN (
                    SELECT MAX(id) FROM trader_order
                    WHERE order_type = 'LIMIT' AND position_type = 'LONG'
                    GROUP BY uuid
                )
                AND order_status <> 'FILLED' AND order_status <> 'CANCELLED'  AND order_status <> 'LIQUIDATE'
            ), commands AS (
                SELECT MAX(id) as id,uuid FROM sorted_set_command
                WHERE uuid IN ( SELECT uuid FROM orders )
                GROUP BY uuid
            ), updates AS (
                SELECT * FROM sorted_set_command
                WHERE id IN ( SELECT id FROM commands )
            ), updated AS(
                SELECT
                    orders.uuid as uuid,
                    COALESCE(amount, entryprice) as entryprice,
                    positionsize as positionsize,
                    updates.command as command
                FROM orders
                LEFT JOIN updates
                ON updates.uuid = orders.uuid
            ), sorted_set AS (
                SELECT *
                FROM sorted_set_command
                WHERE id IN (
                    SELECT MAX(id) FROM sorted_set_command
                    WHERE uuid IS NOT NULL
                    GROUP BY uuid
                )
                AND command IN ('ADD_CLOSE_LIMIT_PRICE', 'UPDATE_CLOSE_LIMIT_PRICE')
                AND position_type ='SHORT'
                ORDER BY id DESC
            )
            SELECT
                trader_o.uuid AS uuid,
                sort.amount AS entryprice,
                trader_o.positionsize AS positionsize
            FROM (
                SELECT *
                FROM trader_order
                WHERE id IN (
                    SELECT MAX(id) FROM trader_order
                    WHERE position_type = 'SHORT'
                    AND uuid IN ( SELECT uuid FROM sorted_set)
                    GROUP BY uuid
                )
                AND trader_order.order_status = 'FILLED'
            ) AS trader_o
            LEFT OUTER JOIN (
                SELECT amount,uuid FROM sorted_set
            ) AS sort
            ON trader_o.uuid = sort.uuid
            UNION ALL
            SELECT
                MAX(uuid) AS uuid,
                entryprice,
                SUM(positionsize) AS positionsize
            FROM updated
            WHERE command IS NULL OR command <> 'REMOVE_CLOSE_LIMIT_PRICE'
            GROUP BY entryprice
            ORDER BY entryprice DESC
            LIMIT 15;
        "#;

        let longs: Vec<OrderBookOrder> = diesel::sql_query(query).get_results(conn)?;

        let bid = longs
            .into_iter()
            .map(|order| Bid {
                id: order.uuid,
                positionsize: order.positionsize.to_f64().unwrap(),
                price: order.entryprice.to_f64().unwrap(),
            })
            .collect();

        let ob = OrderBook { bid, ask };

        Ok(ob)
    }

    pub fn open_orders(conn: &mut PgConnection, customer_id: i64) -> QueryResult<Vec<TraderOrder>> {
        use crate::database::schema::address_customer_id::dsl as acct_dsl;
        // use crate::database::schema::trader_order::dsl::*;

        let account: Vec<AddressCustomerId> = acct_dsl::address_customer_id
            .filter(acct_dsl::customer_id.eq(customer_id))
            .load(conn)?;

        let iter = account.into_iter().map(|a| format!("'{}'", a.address));
        let accounts = join(iter, ", ");

        let query = format!(
            r#"select * from trader_order
            inner join (
                select uuid,max(id) as latest_id
                from trader_order
                group by uuid
            ) mo
            on id = latest_id
            where 
            account_id IN ({})
            and
            order_status IN ('PENDING', 'FILLED')
        "#,
            accounts
        );

        diesel::sql_query(query).get_results(conn)
    }

    pub fn list_past_24hrs(conn: &mut PgConnection) -> QueryResult<Vec<RecentOrder>> {
        // use crate::database::schema::trader_order::dsl::*;

        let query = r#"SELECT * FROM (SELECT
            trader_order.uuid as order_id,
            trader_order.position_type as side,
            trader_order.entryprice as price,
            trader_order.positionsize as positionsize,
            trader_order.timestamp as timestamp
            FROM trader_order
            INNER JOIN (
                SELECT uuid,min(timestamp) AS timestamp
                FROM trader_order  WHERE trader_order.order_status = 'FILLED' and timestamp > now() - INTERVAL '1 day' GROUP BY uuid order by timestamp desc limit 50

            ) as t
            ON trader_order.uuid = t.uuid AND trader_order.timestamp = t.timestamp

            UNION ALL
				
            SELECT
                trader_order.uuid as order_id,
                (
                    CASE WHEN trader_order.position_type = 'LONG' THEN position_type('SHORT')
                    ELSE position_type('LONG')
                    END
                ) as side,
                trader_order.settlement_price as price,
                trader_order.positionsize as positionsize,
                trader_order.timestamp as timestamp

            FROM trader_order
            INNER JOIN (
                SELECT uuid,max(timestamp) AS timestamp
                FROM trader_order 
				where order_status IN ('SETTLED', 'LIQUIDATE')
				AND timestamp > now() - INTERVAL '1 day'
				GROUP BY uuid order by timestamp desc limit 50
			
            ) as t
            ON trader_order.uuid = t.uuid AND trader_order.timestamp = t.timestamp
            order by timestamp desc  ) as recent_order order by timestamp desc limit 50
        "#;

        diesel::sql_query(query).load(conn)
    }
}

impl TraderOrderFundingUpdates {
    pub fn insert(
        conn: &mut PgConnection,
        orders: Vec<InsertTraderOrderFundingUpdates>,
    ) -> QueryResult<usize> {
        use crate::database::schema::trader_order_funding_updated::dsl::*;

        let query = diesel::insert_into(trader_order_funding_updated).values(&orders);

        query.execute(conn)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderBook {
    pub bid: Vec<Bid>,
    pub ask: Vec<Ask>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ask {
    pub id: String,
    pub positionsize: f64,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bid {
    pub id: String,
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
    pub fn get(
        conn: &mut PgConnection,
        customer_id: i64,
        params: OrderId,
    ) -> QueryResult<LendOrder> {
        use crate::database::schema::address_customer_id::dsl as acct_dsl;
        use crate::database::schema::lend_order::dsl::*;

        let accounts: Vec<AddressCustomerId> = acct_dsl::address_customer_id
            .filter(acct_dsl::customer_id.eq(customer_id))
            .load(conn)?;
        let accounts: Vec<_> = accounts.into_iter().map(|a| a.address).collect();

        lend_order
            .filter(uuid.eq(params.id).and(account_id.eq_any(accounts)))
            .order(timestamp.desc())
            .first(conn)
    }
    pub fn get_by_signature(conn: &mut PgConnection, accountid: String) -> QueryResult<LendOrder> {
        use crate::database::schema::lend_order::dsl::*;

        lend_order
            .filter(account_id.eq(accountid))
            .order(timestamp.desc())
            .first(conn)
    }
    pub fn get_by_uuid(conn: &mut PgConnection, order_id: String) -> QueryResult<LendOrder> {
        use crate::database::schema::lend_order::dsl::*;

        lend_order
            .filter(uuid.eq(order_id))
            .order(timestamp.desc())
            .first(conn)
    }

    pub fn insert(conn: &mut PgConnection, orders: Vec<InsertLendOrder>) -> QueryResult<usize> {
        use crate::database::schema::lend_order::dsl::*;

        let query = diesel::insert_into(lend_order).values(&orders);

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
            id: 0,
            uuid: uuid.to_string(),
            account_id,
            position_type: position_type.into(),
            order_status: order_status.into(),
            order_type: order_type.into(),
            // TODO: maybe a TryFrom impl instead...
            entryprice: BigDecimal::from_f64(entryprice).unwrap().round(2),
            execution_price: BigDecimal::from_f64(execution_price).unwrap().round(2),
            positionsize: BigDecimal::from_f64(positionsize).unwrap(),
            leverage: BigDecimal::from_f64(leverage).unwrap(),
            initial_margin: BigDecimal::from_f64(initial_margin).unwrap(),
            available_margin: BigDecimal::from_f64(available_margin).unwrap().round(4),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
            bankruptcy_price: BigDecimal::from_f64(bankruptcy_price).unwrap().round(2),
            bankruptcy_value: BigDecimal::from_f64(bankruptcy_value).unwrap().round(4),
            maintenance_margin: BigDecimal::from_f64(maintenance_margin).unwrap().round(4),
            liquidation_price: BigDecimal::from_f64(liquidation_price).unwrap().round(2),
            unrealized_pnl: BigDecimal::from_f64(unrealized_pnl).unwrap().round(2),
            settlement_price: BigDecimal::from_f64(settlement_price).unwrap().round(2),
            entry_nonce: entry_nonce as i64,
            exit_nonce: exit_nonce as i64,
            entry_sequence: entry_sequence as i64,
        }
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
            entryprice: BigDecimal::from_f64(entryprice).unwrap().round(2),
            execution_price: BigDecimal::from_f64(execution_price).unwrap().round(2),
            positionsize: BigDecimal::from_f64(positionsize).unwrap(),
            leverage: BigDecimal::from_f64(leverage).unwrap(),
            initial_margin: BigDecimal::from_f64(initial_margin).unwrap(),
            available_margin: BigDecimal::from_f64(available_margin).unwrap().round(4),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
            bankruptcy_price: BigDecimal::from_f64(bankruptcy_price).unwrap().round(2),
            bankruptcy_value: BigDecimal::from_f64(bankruptcy_value).unwrap().round(4),
            maintenance_margin: BigDecimal::from_f64(maintenance_margin).unwrap().round(4),
            liquidation_price: BigDecimal::from_f64(liquidation_price).unwrap().round(2),
            unrealized_pnl: BigDecimal::from_f64(unrealized_pnl).unwrap().round(2),
            settlement_price: BigDecimal::from_f64(settlement_price).unwrap().round(2),
            entry_nonce: entry_nonce as i64,
            exit_nonce: exit_nonce as i64,
            entry_sequence: entry_sequence as i64,
        }
    }
}
impl From<relayer::TraderOrder> for TraderOrderFundingUpdates {
    fn from(src: relayer::TraderOrder) -> TraderOrderFundingUpdates {
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

        TraderOrderFundingUpdates {
            id: 0,
            uuid: uuid.to_string(),
            account_id,
            position_type: position_type.into(),
            order_status: order_status.into(),
            order_type: order_type.into(),
            // TODO: maybe a TryFrom impl instead...
            entryprice: BigDecimal::from_f64(entryprice).unwrap().round(2),
            execution_price: BigDecimal::from_f64(execution_price).unwrap().round(2),
            positionsize: BigDecimal::from_f64(positionsize).unwrap(),
            leverage: BigDecimal::from_f64(leverage).unwrap(),
            initial_margin: BigDecimal::from_f64(initial_margin).unwrap(),
            available_margin: BigDecimal::from_f64(available_margin).unwrap().round(4),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
            bankruptcy_price: BigDecimal::from_f64(bankruptcy_price).unwrap().round(2),
            bankruptcy_value: BigDecimal::from_f64(bankruptcy_value).unwrap().round(4),
            maintenance_margin: BigDecimal::from_f64(maintenance_margin).unwrap().round(4),
            liquidation_price: BigDecimal::from_f64(liquidation_price).unwrap().round(2),
            unrealized_pnl: BigDecimal::from_f64(unrealized_pnl).unwrap().round(2),
            settlement_price: BigDecimal::from_f64(settlement_price).unwrap().round(2),
            entry_nonce: entry_nonce as i64,
            exit_nonce: exit_nonce as i64,
            entry_sequence: entry_sequence as i64,
        }
    }
}
impl From<relayer::TraderOrder> for InsertTraderOrderFundingUpdates {
    fn from(src: relayer::TraderOrder) -> InsertTraderOrderFundingUpdates {
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

        InsertTraderOrderFundingUpdates {
            uuid: uuid.to_string(),
            account_id,
            position_type: position_type.into(),
            order_status: order_status.into(),
            order_type: order_type.into(),
            // TODO: maybe a TryFrom impl instead...
            entryprice: BigDecimal::from_f64(entryprice).unwrap().round(2),
            execution_price: BigDecimal::from_f64(execution_price).unwrap().round(2),
            positionsize: BigDecimal::from_f64(positionsize).unwrap(),
            leverage: BigDecimal::from_f64(leverage).unwrap(),
            initial_margin: BigDecimal::from_f64(initial_margin).unwrap(),
            available_margin: BigDecimal::from_f64(available_margin).unwrap().round(4),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
            bankruptcy_price: BigDecimal::from_f64(bankruptcy_price).unwrap().round(2),
            bankruptcy_value: BigDecimal::from_f64(bankruptcy_value).unwrap().round(4),
            maintenance_margin: BigDecimal::from_f64(maintenance_margin).unwrap().round(4),
            liquidation_price: BigDecimal::from_f64(liquidation_price).unwrap().round(2),
            unrealized_pnl: BigDecimal::from_f64(unrealized_pnl).unwrap().round(2),
            settlement_price: BigDecimal::from_f64(settlement_price).unwrap().round(2),
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
            new_lend_state_amount: BigDecimal::from_f64(new_lend_state_amount)
                .unwrap()
                .round(4),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .expect("Bad datetime format")
                .into(),
            npoolshare: BigDecimal::from_f64(npoolshare).unwrap().round(4),
            nwithdraw: BigDecimal::from_f64(nwithdraw).unwrap().round(4),
            payment: BigDecimal::from_f64(payment).unwrap().round(4),
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

    // fn make_trader_order(entryprice: f64, execution_price: f64) -> TraderOrder {
    //     let mut bytes = [0u8; 16];

    //     getrandom(&mut bytes).expect("Could not get randomness");

    //     TraderOrder {
    //         uuid: bytes.encode_hex::<String>(),
    //         account_id: "my-id".into(),
    //         position_type: PositionType::LONG,
    //         order_status: OrderStatus::PENDING,
    //         order_type: OrderType::MARKET,
    //         entryprice: BigDecimal::from_f64(entryprice).unwrap(),
    //         execution_price: BigDecimal::from_f64(execution_price).unwrap(),
    //         positionsize: BigDecimal::from_f64(0.0).unwrap(),
    //         leverage: BigDecimal::from_f64(0.0).unwrap(),
    //         initial_margin: BigDecimal::from_f64(0.0).unwrap(),
    //         available_margin: BigDecimal::from_f64(0.0).unwrap(),
    //         timestamp: Utc::now(),
    //         bankruptcy_price: BigDecimal::from_f64(0.0).unwrap(),
    //         bankruptcy_value: BigDecimal::from_f64(0.0).unwrap(),
    //         maintenance_margin: BigDecimal::from_f64(0.0).unwrap(),
    //         liquidation_price: BigDecimal::from_f64(0.0).unwrap(),
    //         unrealized_pnl: BigDecimal::from_f64(0.0).unwrap(),
    //         settlement_price: BigDecimal::from_f64(0.0).unwrap(),
    //         entry_nonce: 20,
    //         exit_nonce: 22,
    //         entry_sequence: 400,
    //     }
    // }

    // fn make_lend_order(balance: f64, payment: f64) -> LendOrder {
    //     let mut bytes = [0u8; 16];

    //     getrandom(&mut bytes).expect("Could not get randomness");

    //     LendOrder {
    //         uuid: bytes.encode_hex::<String>(),
    //         account_id: "lender-id".into(),
    //         balance: BigDecimal::from_f64(balance).unwrap(),
    //         order_status: OrderStatus::PENDING,
    //         order_type: OrderType::MARKET,
    //         entry_nonce: 40,
    //         exit_nonce: 600,
    //         deposit: BigDecimal::from_f64(0.0).unwrap(),
    //         new_lend_state_amount: BigDecimal::from_f64(0.0).unwrap(),
    //         timestamp: Utc::now(),
    //         npoolshare: BigDecimal::from_f64(0.0).unwrap(),
    //         nwithdraw: BigDecimal::from_f64(0.0).unwrap(),
    //         payment: BigDecimal::from_f64(payment).unwrap(),
    //         tlv0: BigDecimal::from_f64(0.0).unwrap(),
    //         tps0: BigDecimal::from_f64(0.0).unwrap(),
    //         tlv1: BigDecimal::from_f64(0.0).unwrap(),
    //         tps1: BigDecimal::from_f64(0.0).unwrap(),
    //         tlv2: BigDecimal::from_f64(0.0).unwrap(),
    //         tps2: BigDecimal::from_f64(0.0).unwrap(),
    //         tlv3: BigDecimal::from_f64(0.0).unwrap(),
    //         tps3: BigDecimal::from_f64(0.0).unwrap(),
    //         entry_sequence: 0,
    //     }
    // }

    // #[test]
    // fn trader_orders() {
    //     use crate::database::schema::trader_order::dsl::*;

    //     let mut conn =
    //         PgConnection::establish(DIESEL_TEST_URL).expect("Could not establish test connection!");

    //     conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
    //         let mut order1 = make_trader_order(1.0, 4.0);
    //         let mut order2 = make_trader_order(4.0, 400.0);

    //         let orders: Vec<TraderOrder> = vec![order1.clone(), order2.clone()];

    //         let result = diesel::insert_into(trader_order)
    //             .values(orders)
    //             .execute(&mut *conn);

    //         if let Err(e) = result {
    //             panic!("insert in database didn't suceed! {:#?}", e);
    //         }

    //         //Test updates/inserts
    //         let order3 = make_trader_order(989.0, 23.0);
    //         let order4 = make_trader_order(99.0, 302.0);

    //         order1.entryprice = BigDecimal::from_f64(32.0).unwrap();
    //         order1.execution_price = BigDecimal::from_f64(89.0).unwrap();
    //         order2.entryprice = BigDecimal::from_f64(20.0).unwrap();

    //         TraderOrder::update_or_insert(
    //             &mut *conn,
    //             vec![
    //                 order1.clone(),
    //                 order2.clone(),
    //                 order3.clone(),
    //                 order4.clone(),
    //             ],
    //         )?;

    //         let o1: TraderOrder = trader_order
    //             .filter(uuid.eq(order1.uuid))
    //             .first(&mut *conn)?;

    //         assert_eq!(o1.entryprice, order1.entryprice);
    //         assert_eq!(o1.execution_price, order1.execution_price);

    //         let o2: TraderOrder = trader_order
    //             .filter(uuid.eq(order2.uuid))
    //             .first(&mut *conn)?;

    //         assert_eq!(o2.entryprice, order2.entryprice);
    //         assert_eq!(o2.execution_price, order2.execution_price);

    //         Ok(())
    //     });
    // }

    // #[test]
    // fn lender_orders() {
    //     use crate::database::schema::lend_order::dsl::*;

    //     let mut conn =
    //         PgConnection::establish(DIESEL_TEST_URL).expect("Could not establish test connection!");

    //     conn.test_transaction::<_, diesel::result::Error, _>(|conn| {
    //         let mut order1 = make_lend_order(1.0, 4.0);
    //         let mut order2 = make_lend_order(4.0, 400.0);

    //         let orders: Vec<LendOrder> = vec![order1.clone(), order2.clone()];

    //         let result = diesel::insert_into(lend_order)
    //             .values(orders)
    //             .execute(&mut *conn);

    //         if let Err(e) = result {
    //             panic!("insert in database didn't suceed! {:#?}", e);
    //         }

    //         //Test updates/inserts
    //         let order3 = make_lend_order(989.0, 23.0);
    //         let order4 = make_lend_order(99.0, 302.0);

    //         order1.balance = BigDecimal::from_f64(32.0).unwrap();
    //         order1.payment = BigDecimal::from_f64(89.0).unwrap();
    //         order2.balance = BigDecimal::from_f64(20.0).unwrap();

    //         LendOrder::update_or_insert(
    //             &mut *conn,
    //             vec![
    //                 order1.clone(),
    //                 order2.clone(),
    //                 order3.clone(),
    //                 order4.clone(),
    //             ],
    //         )?;

    //         let o1: LendOrder = lend_order.filter(uuid.eq(order1.uuid)).first(&mut *conn)?;

    //         assert_eq!(o1.balance, order1.balance);
    //         assert_eq!(o1.payment, order1.payment);

    //         let o2: LendOrder = lend_order.filter(uuid.eq(order2.uuid)).first(&mut *conn)?;

    //         assert_eq!(o2.balance, order2.balance);
    //         assert_eq!(o2.payment, order2.payment);

    //         Ok(())
    //     });
    // }
}
