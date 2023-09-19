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
        pub account_address: String,
    }
}
