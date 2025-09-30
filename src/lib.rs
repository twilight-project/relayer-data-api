mod archiver;
pub mod database;
pub mod error;
pub mod kafka;
pub(crate) mod migrations;
pub mod rpc;
pub mod ws;
pub extern crate relayer_core;
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
