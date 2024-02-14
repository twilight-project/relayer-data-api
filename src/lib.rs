mod archiver;
pub mod database;
pub mod error;
pub mod kafka;
pub(crate) mod migrations;
pub mod rpc;
pub mod ws;

pub use archiver::DatabaseArchiver;

pub mod auth {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct AuthInfo {
        pub api_key: String,
        pub api_secret: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct UserInfo {
        pub customer_id: i64,
    }
}

// for openrpc api
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;
const MAX_RETRIES: usize = 5;
const RETRY_SLEEP: u64 = 2000;
use crate::error::ApiError;
use log::{debug, error, info, trace};
use r2d2::PooledConnection;
use std::time::{Duration, Instant};
pub struct RelayerDB {
    pool: ManagedPool,
}
impl RelayerDB {
    pub fn from_host(database_url: String) -> Self {
        RelayerDB {
            pool: {
                let manager = ConnectionManager::<PgConnection>::new(database_url);
                let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");
                pool
            },
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
}

#[cfg(test)]
mod test {

    use crate::{database::TraderOrder as TraderOrderDB, RelayerDB};
    #[test]
    fn test_check_get_order_by_uuid_from_archiver() {
        dotenv::dotenv().expect("Failed loading dotenv");
        let database_url = std::env::var("DATABASE_URL").expect("No database url found!");
        let relayer_db = RelayerDB::from_host(database_url);
        let mut pool = relayer_db.get_conn().unwrap();
        TraderOrderDB::get_by_uuid(&mut *pool, "".to_string());
    }
}
