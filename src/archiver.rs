use crate::{database::*, error::ApiError, migrations};
use crossbeam_channel::Receiver;
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use log::{debug, error, info};
use r2d2::PooledConnection;
use std::time::{Duration, Instant};
use twilight_relayer_rust::db::{Event, EventLog};

const BATCH_INTERVAL: u64 = 100;
const BATCH_SIZE: usize = 500;
const MAX_RETRIES: usize = 5;
const RETRY_SLEEP: u64 = 2000;

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

pub struct DatabaseArchiver {
    pool: ManagedPool,
    trader_orders: Vec<TraderOrder>,
    lend_orders: Vec<LendOrder>,
}

impl DatabaseArchiver {
    /// Start an archiver, provided a postgres connection string.
    pub fn from_host(database_url: String) -> DatabaseArchiver {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");

        let mut conn = pool.get().expect("Could not get pooled connection!");

        migrations::run_migrations(&mut *conn).expect("Failed to run database migrations!");

        let trader_orders = Vec::with_capacity(BATCH_SIZE);
        let lend_orders = Vec::with_capacity(BATCH_SIZE);

        DatabaseArchiver {
            pool,
            trader_orders,
            lend_orders,
        }
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

    /// Add a trader order to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn trader_order(&mut self, order: TraderOrder) -> Result<(), ApiError> {
        debug!("Appending trader order");
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

        TraderOrder::update_or_insert(&mut conn, orders)?;

        Ok(())
    }

    /// Add a lend order to the next update batch, if the queue is full, commit and clear the
    /// queue.
    fn lend_order(&mut self, order: LendOrder) -> Result<(), ApiError> {
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

        LendOrder::update_or_insert(&mut conn, orders)?;

        Ok(())
    }

    /// Commit any pending orders of any type, regardless of batch size.
    fn commit_orders(&mut self) -> Result<(), ApiError> {
        if self.trader_orders.len() > 0 {
            self.commit_trader_orders()?;
        }

        if self.lend_orders.len() > 0 {
            self.commit_lend_orders()?;
        }

        //TODO: other order types...

        Ok(())
    }

    /// Worker task that loops indefinitely, batching commits to postgres backend.
    pub fn run(mut self, rx: Receiver<EventLog>) -> Result<(), ApiError> {
        let mut deadline = Instant::now() + Duration::from_millis(BATCH_INTERVAL);

        loop {
            match rx.recv_deadline(deadline) {
                Ok(msg) => {
                    let EventLog {
                        offset: _,
                        key: _,
                        value,
                    } = msg;
                    match value {
                        Event::TraderOrder(trader_order, ..) => {
                            self.trader_order(trader_order.into())?
                        }
                        Event::TraderOrderUpdate(trader_order, ..) => {
                            self.trader_order(trader_order.into())?
                        }
                        Event::TraderOrderFundingUpdate(trader_order, ..) => {
                            self.trader_order(trader_order.into())?
                        }
                        Event::TraderOrderLiquidation(trader_order, ..) => {
                            self.trader_order(trader_order.into())?
                        }
                        Event::LendOrder(lend_order, ..) => self.lend_order(lend_order.into())?,
                        Event::FundingRateUpdate(funding_rate, _system_time) => {
                            FundingRateUpdate::insert(&mut *self.get_conn()?, funding_rate)?;
                        }
                        Event::CurrentPriceUpdate(current_price, _system_time) => {
                            CurrentPriceUpdate::insert(&mut *self.get_conn()?, current_price)?;
                        }
                        Event::PoolUpdate(_lend_pool_command, ..) => {
                            info!("FINISH POOL UPDATE");
                        }
                        Event::SortedSetDBUpdate(_sorted_set_command) => {
                            info!("FINISH SORTED SET DB UPDATE");
                        }
                        Event::PositionSizeLogDBUpdate(
                            _position_size_log_command,
                            _position_size_log,
                        ) => {
                            info!("FINISH POSITION SIZE LOG DB UPDATE");
                        }
                        Event::Stop(_stop) => {
                            info!("FINISH STOP");
                        }
                    }
                }
                Err(e) => {
                    if e.is_timeout() {
                        debug!("Timeout reached, committing current orders");
                        self.commit_orders()?;

                        deadline = Instant::now() + Duration::from_millis(BATCH_INTERVAL);
                        debug!("New deadline: {:?}", deadline);
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
