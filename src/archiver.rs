use bigdecimal::ToPrimitive;
use crate::{database::*, error::ApiError, kafka::Completion, migrations};
use chrono::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use log::{debug, error, info, trace};
use r2d2::PooledConnection;
use redis::Client;
use serde_json::json;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use twilight_relayer_rust::{
    db::{self as relayer_db, Event},
    relayer,
};

const BATCH_INTERVAL: u64 = 100;
const BATCH_SIZE: usize = 2_000;
const MAX_RETRIES: usize = 5;
const RETRY_SLEEP: u64 = 2000;
const PIPELINE_CHUNK: usize = 512;

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

const UPDATE_FN: &str = r#"
    -- args: <order_id> <order_status> <side> <price> <position_size> <rfc3339> <timestamp_millis>
    local id = ARGV[1]
    local status = ARGV[2]
    local side = ARGV[3]
    local price = tonumber(ARGV[4])
    local size = tonumber(ARGV[5])
    local timestamp = ARGV[6]
    local time = tonumber(ARGV[7])

    if status == "FILLED" or status == "SETTLED" or status == "LIQUIDATE" then
        redis.call('HDEL', 'orders', id)

        local table = { order_id = id, side = side, price = price, positionsize = size, timestamp = timestamp }

        local order_json = cjson.encode(table)
        redis.call('ZADD', 'recent_orders', time, order_json)

        local result = tonumber(redis.pcall('ZRANGEBYSCORE', side, price, price)[1])
        local new_size = result - size

        redis.call('ZREM', side, result)
        redis.call('ZADD', side, price, new_size)

        return
    end

    -- just opened a new order
    if status == "PENDING" then
        redis.call('HSET', 'orders', id, price)

        local result = tonumber(redis.pcall('ZRANGEBYSCORE', side, price, price)[1])
        local new_size = result + size

        redis.call('ZREM', side, result)
        redis.call('ZADD', side, price, new_size)
    end

    -- TODO: clean out <recent_orders> expired > 24h...
"#;

pub struct DatabaseArchiver {
    redis: Client,
    pool: ManagedPool,
    script_sha: String,
    trader_orders: Vec<InsertTraderOrder>,
    trader_order_funding_updated: Vec<InsertTraderOrderFundingUpdates>,
    lend_orders: Vec<InsertLendOrder>,
    position_size: Vec<PositionSizeUpdate>,
    tx_hashes: Vec<NewTxHash>,
    sorted_set: Vec<relayer::SortedSetCommand>,
    lend_pool: Vec<relayer_db::LendPool>,
    lend_pool_commands: Vec<relayer_db::LendPoolCommand>,
    completions: Sender<Completion>,
    nonce: Nonce,
}

impl DatabaseArchiver {
    /// Start an archiver, provided a postgres connection string.
    pub fn from_host(
        database_url: &str,
        redis_url: &str,
        completions: Sender<Completion>,
    ) -> DatabaseArchiver {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");
        let redis = Client::open(redis_url).expect("Could not establish redis connection");

        let mut conn = pool.get().expect("Could not get pooled connection!");

        migrations::run_migrations(&mut *conn).expect("Failed to run database migrations!");

        let trader_orders = Vec::with_capacity(BATCH_SIZE);
        let trader_order_funding_updated = Vec::with_capacity(BATCH_SIZE);
        let lend_orders = Vec::with_capacity(BATCH_SIZE);
        let position_size = Vec::with_capacity(BATCH_SIZE);
        let tx_hashes = Vec::with_capacity(BATCH_SIZE);
        let sorted_set = Vec::with_capacity(BATCH_SIZE);
        let lend_pool = Vec::with_capacity(BATCH_SIZE);
        let lend_pool_commands = Vec::with_capacity(BATCH_SIZE);
        let nonce = Nonce::get(&mut conn).expect("Failed to query for current nonce");

        Self::load_cache(pool.clone(), redis.clone());

        let mut redis_conn = redis.get_connection().expect("Redis connection");
        let script_sha: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(UPDATE_FN)
            .query(&mut redis_conn)
            .expect("Script load failed");

        DatabaseArchiver {
            redis,
            pool,
            script_sha,
            trader_orders,
            trader_order_funding_updated,
            lend_orders,
            position_size,
            tx_hashes,
            sorted_set,
            lend_pool,
            lend_pool_commands,
            completions,
            nonce,
        }
    }

    fn load_cache(pool: ManagedPool, redis: Client) {
        let mut conn = pool.get().expect("Could not get pooled connection!");

        let recent_orders =
            TraderOrder::list_past_24hrs(&mut conn).expect("Failed to load recent orders");
        let order_book_orders = TraderOrder::order_book_orders(&mut conn).expect("Failed to load order book");

        for chunk in recent_orders.chunks(PIPELINE_CHUNK) {
            let mut redis_conn = redis
                .get_connection()
                .expect("Failed to acquire redis connection");
            let mut pipe = redis::pipe();

            for item in chunk {
                let timestamp = item.timestamp.timestamp_millis();

                let order = json!({
                    "order_id": item.order_id.clone(),
                    "side": item.side.to_string(),
                    "price": item.price,
                    "positionsize": item.positionsize,
                    "timestamp": item.timestamp.to_rfc3339(),
                });

                let order = serde_json::to_string(&order).expect("Invalid JSON");

                let mut cmd = redis::cmd("ZADD");
                cmd.arg("recent_orders").arg(timestamp).arg(order);
                pipe.add_command(cmd);
            }

            pipe.atomic().execute(&mut redis_conn);
        }

        let mut redis_conn = redis
            .get_connection()
            .expect("Failed to acquire redis connection");
        let mut bids = HashMap::new();
        let mut asks = HashMap::new();

        for order in order_book_orders.iter() {
            let key = (order.entryprice.to_f64().unwrap() * 100.0) as i64;
            let positionsize = order.positionsize.to_f64().unwrap();

            if order.position_type == PositionType::SHORT {
                asks.entry(key)
                    .and_modify(|size| *size += positionsize)
                    .or_insert(positionsize);
            } else {
                bids.entry(key)
                    .and_modify(|size| *size += positionsize)
                    .or_insert(positionsize);
            }

            redis::cmd("HSET")
                .arg("orders")
                .arg("id")
                .arg(order.uuid.clone())
                .arg("price")
                .arg(order.entryprice.to_f64())
                .execute(&mut redis_conn);
        }

        for (price, size) in bids.into_iter() {
            redis::cmd("ZADD")
                .arg("bid")
                .arg(price)
                .arg(size.to_f64())
                .execute(&mut redis_conn);
        }

        for (price, size) in asks.into_iter() {
            redis::cmd("ZADD")
                .arg("ask")
                .arg(price)
                .arg(size.to_f64())
                .execute(&mut redis_conn);
        }
    }

    fn update_sorted_set(&self, cmd: &relayer::SortedSetCommand) -> Result<(), ApiError> {
        Ok(())
    }

    fn update_order_cache(&self, order: &InsertTraderOrder) -> Result<(), ApiError> {
        let mut pipe = redis::pipe();

        let side = match order.position_type {
            PositionType::LONG => "bid",
            PositionType::SHORT => "ask",
        };

        let mut cmd = redis::cmd("EVALSHA");
        cmd.arg(&self.script_sha)
            .arg(order.uuid.clone())
            .arg(order.order_status.as_str())
            .arg(side)
            .arg(order.entryprice.to_string())
            .arg(order.positionsize.to_string())
            .arg(order.timestamp.to_rfc3339())
            .arg(order.timestamp.timestamp_millis());

        pipe.add_command(cmd);
        let mut redis_conn = self.redis.get_connection()?;
        pipe.atomic().execute(&mut redis_conn);
        Ok(())
    }

    /// Fetch a connection, will retry MAX_RETRIES before giving up.
    fn get_conn(&self) -> Result<PooledConnection<ManagedConnection>, ApiError> {
        let mut retries = MAX_RETRIES;

        Ok(loop {
            break match self.pool.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("Could not get connection from connection pool! {:?}", e);
                    std::thread::sleep(Duration::from_millis(RETRY_SLEEP));

                    if retries == 0 {
                        return Err(ApiError::CommitRetryCountExceeded);
                    }

                    retries -= 1;

                    continue;
                }
            };
        })
    }

    /// Add a sorted set update to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn sorted_set_update(
        &mut self,
        sorted_set_update: relayer::SortedSetCommand,
    ) -> Result<(), ApiError> {
        debug!("Appending sorted set update");
        let _ = self.update_sorted_set(&sorted_set_update);
        self.sorted_set.push(sorted_set_update);

        if self.sorted_set.len() == self.sorted_set.capacity() {
            self.commit_sorted_set_updates()?;
        }

        Ok(())
    }

    /// Commit a batch of sorted set updates to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_sorted_set_updates(&mut self) -> Result<(), ApiError> {
        debug!("Committing sorted sets");

        let mut conn = self.get_conn()?;

        let mut updates = Vec::with_capacity(self.sorted_set.capacity());
        std::mem::swap(&mut updates, &mut self.sorted_set);

        SortedSetCommand::append(&mut conn, updates)?;

        Ok(())
    }

    /// Add a position size update to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn position_size_log(
        &mut self,
        position_size_update: PositionSizeUpdate,
    ) -> Result<(), ApiError> {
        debug!("Appending position size update");
        self.position_size.push(position_size_update);

        if self.position_size.len() == self.position_size.capacity() {
            self.commit_position_sizes()?;
        }

        Ok(())
    }

    /// Commit a batch of position sizes to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_position_sizes(&mut self) -> Result<(), ApiError> {
        debug!("Committing position sizes");

        let mut conn = self.get_conn()?;

        let mut sizes = Vec::with_capacity(self.position_size.capacity());
        std::mem::swap(&mut sizes, &mut self.position_size);

        PositionSizeLog::append(&mut conn, sizes)?;

        Ok(())
    }

    fn tx_hash(&mut self, hash: NewTxHash) -> Result<(), ApiError> {
        debug!("Appending position size update");
        self.tx_hashes.push(hash);

        if self.tx_hashes.len() == self.tx_hashes.capacity() {
            self.commit_tx_hash()?;
        }

        Ok(())
    }

    /// Commit a batch of tx hashes to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_tx_hash(&mut self) -> Result<(), ApiError> {
        debug!("Committing tx hashes");

        let mut conn = self.get_conn()?;

        let mut hashes = Vec::with_capacity(self.tx_hashes.capacity());
        std::mem::swap(&mut hashes, &mut self.tx_hashes);

        TxHash::append(&mut conn, hashes)?;

        Ok(())
    }
    /// Add a trader order to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn trader_order(&mut self, order: InsertTraderOrder) -> Result<(), ApiError> {
        debug!("Appending trader order");
        let _ = self.update_order_cache(&order);
        self.trader_orders.push(order);

        if self.trader_orders.len() == self.trader_orders.capacity() {
            self.commit_trader_orders()?;
        }

        Ok(())
    }

    /// Commit a batch of trader orders to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_trader_orders(&mut self) -> Result<(), ApiError> {
        debug!("Committing trader orders");

        let mut conn = self.get_conn()?;

        let mut orders = Vec::with_capacity(self.trader_orders.capacity());
        std::mem::swap(&mut orders, &mut self.trader_orders);

        TraderOrder::insert(&mut conn, orders)?;

        Ok(())
    }
    /// Add a trader order funidng update to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn trader_order_funding_update(
        &mut self,
        order: InsertTraderOrderFundingUpdates,
    ) -> Result<(), ApiError> {
        debug!("Appending trader order");
        self.trader_order_funding_updated.push(order);

        if self.trader_order_funding_updated.len() == self.trader_order_funding_updated.capacity() {
            self.commit_trader_order_funding_updated()?;
        }

        Ok(())
    }

    /// Commit a batch of trader orders funidng update to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_trader_order_funding_updated(&mut self) -> Result<(), ApiError> {
        debug!("Committing trader orders");

        let mut conn = self.get_conn()?;

        let mut orders = Vec::with_capacity(self.trader_order_funding_updated.capacity());
        std::mem::swap(&mut orders, &mut self.trader_order_funding_updated);

        TraderOrderFundingUpdates::insert(&mut conn, orders)?;

        Ok(())
    }

    /// Add a lend order to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn lend_order(&mut self, order: InsertLendOrder) -> Result<(), ApiError> {
        debug!("Appending lend order");
        self.lend_orders.push(order);

        if self.lend_orders.len() == self.lend_orders.capacity() {
            self.commit_lend_orders()?;
        }

        Ok(())
    }

    /// Commit a batch of lend orders to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_lend_orders(&mut self) -> Result<(), ApiError> {
        debug!("Committing lend orders");

        let mut conn = self.get_conn()?;

        let mut orders = Vec::with_capacity(self.lend_orders.capacity());
        std::mem::swap(&mut orders, &mut self.lend_orders);

        LendOrder::insert(&mut conn, orders)?;

        Ok(())
    }

    /// Add a lend pool update to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn lend_pool_updates(&mut self, update: relayer_db::LendPool) -> Result<(), ApiError> {
        debug!("Appending lend pool update");
        self.lend_pool.push(update);

        if self.lend_pool.len() == self.lend_pool.capacity() {
            self.commit_lend_pool()?;
        }

        Ok(())
    }

    /// Commit a batch of lend pool updates to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_lend_pool(&mut self) -> Result<(), ApiError> {
        debug!("Committing lend pool commands");

        let mut conn = self.get_conn()?;

        let mut pool = Vec::with_capacity(self.lend_pool.capacity());
        std::mem::swap(&mut pool, &mut self.lend_pool);

        LendPool::insert(&mut conn, pool)?;

        Ok(())
    }

    /// Add a lend pool update to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn lend_pool_command(&mut self, update: relayer_db::LendPoolCommand) -> Result<(), ApiError> {
        debug!("Appending lend pool update");
        self.lend_pool_commands.push(update);

        if self.lend_pool_commands.len() == self.lend_pool_commands.capacity() {
            self.commit_lend_pool_commands()?;
        }

        Ok(())
    }

    /// Commit a batch of lend pool updates to the database. If we're failing to update the database, we
    /// should exit.
    fn commit_lend_pool_commands(&mut self) -> Result<(), ApiError> {
        debug!("Committing lend pool commands");

        let mut conn = self.get_conn()?;

        let mut pool = Vec::with_capacity(self.lend_pool_commands.capacity());
        std::mem::swap(&mut pool, &mut self.lend_pool_commands);

        LendPoolCommand::insert(&mut conn, pool, &mut self.nonce)?;

        Ok(())
    }

    /// Commit any pending orders of any type, regardless of batch size.
    fn commit_orders(&mut self) -> Result<(), ApiError> {
        if self.trader_orders.len() > 0 {
            self.commit_trader_orders()?;
        }
        if self.trader_order_funding_updated.len() > 0 {
            self.commit_trader_order_funding_updated()?;
        }

        if self.lend_orders.len() > 0 {
            self.commit_lend_orders()?;
        }

        if self.position_size.len() > 0 {
            self.commit_position_sizes()?;
        }

        if self.tx_hashes.len() > 0 {
            self.commit_tx_hash()?;
        }

        if self.sorted_set.len() > 0 {
            self.commit_sorted_set_updates()?;
        }

        if self.lend_pool.len() > 0 {
            self.commit_lend_pool()?;
        }

        if self.lend_pool_commands.len() > 0 {
            self.commit_lend_pool_commands()?;
        }

        Ok(())
    }

    fn process_msg(&mut self, event: Event) -> Result<(), ApiError> {
        match event {
            Event::TraderOrder(trader_order, _cmd, _seq) => {
                self.trader_order(trader_order.into())?;
            }
            Event::TraderOrderUpdate(trader_order, _cmd, _seq) => {
                self.trader_order(trader_order.into())?;
            }
            Event::TraderOrderFundingUpdate(trader_order, _cmd) => {
                self.trader_order_funding_update(trader_order.into())?;
            }
            Event::TraderOrderLiquidation(trader_order, _cmd, _seq) => {
                self.trader_order(trader_order.into())?;
            }
            Event::LendOrder(lend_order, _cmd, _seq) => self.lend_order(lend_order.into())?,
            Event::FundingRateUpdate(funding_rate, btc_price, system_time) => {
                let ts = DateTime::parse_from_rfc3339(&system_time)
                    .expect("Bad datetime format")
                    .into();
                FundingRateUpdate::insert(&mut *self.get_conn()?, funding_rate, btc_price, ts)?;
            }
            Event::CurrentPriceUpdate(current_price, system_time) => {
                let ts = DateTime::parse_from_rfc3339(&system_time)
                    .expect("Bad datetime format")
                    .into();
                CurrentPriceUpdate::insert(&mut *self.get_conn()?, current_price, ts)?;
            }
            Event::PoolUpdate(lend_pool_command, lend_pool, ..) => {
                self.lend_pool_updates(lend_pool)?;
                self.lend_pool_command(lend_pool_command)?;
            }
            Event::SortedSetDBUpdate(sorted_set_command) => {
                self.sorted_set_update(sorted_set_command)?;
            }
            Event::PositionSizeLogDBUpdate(position_size_log_command, position_size_log) => {
                self.position_size_log((position_size_log_command, position_size_log))?;
            }
            Event::Stop(_stop) => {
                info!("FINISH STOP");
            }
            Event::TxHash(
                uuid,
                account_id,
                tx_hash,
                order_type,
                order_status,
                datetime,
                output,
                request_id,
            ) => {
                let hash = NewTxHash {
                    order_id: uuid.to_string(),
                    account_id,
                    tx_hash,
                    order_type: order_type.into(),
                    order_status: order_status.into(),
                    datetime,
                    output,
                    request_id: Some(request_id),
                };
                self.tx_hash(hash)?;
            }
            Event::TxHashUpdate(
                uuid,
                account_id,
                tx_hash,
                order_type,
                order_status,
                datetime,
                output,
            ) => {
                let hash = NewTxHash {
                    order_id: uuid.to_string(),
                    account_id,
                    tx_hash,
                    order_type: order_type.into(),
                    order_status: order_status.into(),
                    datetime,
                    output,
                    request_id: None,
                };
                self.tx_hash(hash)?;
            }
            Event::AdvanceStateQueue(_, _) => {}
        }

        Ok(())
    }

    /// Worker task that loops indefinitely, batching commits to postgres backend.
    pub fn run(mut self, rx: Receiver<(Completion, Vec<Event>)>) -> Result<(), ApiError> {
        let mut deadline = Instant::now() + Duration::from_millis(BATCH_INTERVAL);

        loop {
            match rx.recv_deadline(deadline) {
                Ok((completion, msgs)) => {
                    for msg in msgs {
                        self.process_msg(msg)?;
                    }

                    self.completions
                        .send(completion)
                        .map_err(|e| ApiError::CrossbeamChannel(format!("{:?}", e)))?;
                }
                Err(e) => {
                    if e.is_timeout() {
                        trace!("Timeout reached, committing current orders");
                        self.commit_orders()?;

                        deadline = Instant::now() + Duration::from_millis(BATCH_INTERVAL);
                        trace!("New deadline: {:?}", deadline);
                    } else {
                        error!("Channel disconnected!");
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
