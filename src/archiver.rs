use crossbeam_channel::Receiver;
use crate::database::*;
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use log::{error, info};
use r2d2::PooledConnection;
use std::time::Duration;
use twilight_relayer_rust::db::{Event, EventLog};


const RETRY_SLEEP: u64 = 200;

type ManagedPool = r2d2::Pool<ConnectionManager<PgConnection>>;


pub struct DatabaseArchiver {
    database_url: String,
}

impl DatabaseArchiver {
    pub fn from_host(database_url: String) -> DatabaseArchiver {
        DatabaseArchiver {
            database_url
        }
    }

    pub fn run(self, rx: Receiver<EventLog>) {
        let manager = ConnectionManager::<PgConnection>::new(self.database_url);
        let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");

        loop {
            while let Ok(msg) = rx.recv() {
                let EventLog { offset, key, value } = msg;

                // TODO: maybe a retry count before giving up?
                let mut conn = match pool.get() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Could not get connection from connection pool!");
                        std::thread::sleep(Duration::from_millis(RETRY_SLEEP));
                        continue;
                    }
                };

                match value {
                    Event::TraderOrder(trader_order, ..) => {
                        let db_order: TraderOrder = trader_order.into();
                        db_order.insert(&mut *conn);
                    },
                    Event::TraderOrderUpdate(trader_order, ..) => {
                        let db_order: TraderOrder = trader_order.into();
                        db_order.update(&mut *conn);
                    },
                    Event::TraderOrderFundingUpdate(trader_order, ..) => {
                        let db_order: TraderOrder = trader_order.into();
                    },
                    Event::TraderOrderLiquidation(trader_order, ..) => {
                        let db_order: TraderOrder = trader_order.into();
                    },
                    Event::LendOrder(lend_order, ..) => {
                        info!("FINISH LEND ORDER");
                    },
                    Event::PoolUpdate(lend_pool_command, ..) => {
                        info!("FINISH POOL UPDATE");
                    },
                    Event::FundingRateUpdate(_, system_time) => {
                        info!("FINISH FUNDING RATE UPDATE");
                    },
                    Event::CurrentPriceUpdate(_, system_time) => {
                        info!("FINISH CURRENT PRICE UPDATE");
                    },
                    Event::SortedSetDBUpdate(sorted_set_command) => {
                        info!("FINISH SORTED SET DB UPDATE");
                    },
                    Event::PositionSizeLogDBUpdate(position_size_log_command, position_size_log) => {
                        info!("FINISH POSITION SIZE LOG DB UPDATE");
                    },
                    Event::Stop(stop) => {
                        info!("FINISH STOP");
                    },
                }
            }
        }
    }
}
