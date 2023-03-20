use crate::migrations;
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use jsonrpsee::{core::error::Error, server::logger::Params, RpcModule};
use log::info;
use serde::Serialize;
use tokio::{
    sync::broadcast::{channel, Receiver, Sender},
    task::JoinHandle,
};

mod methods;

const BROADCAST_CHANNEL_CAPACITY: usize = 20;

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

pub struct WsContext {
    hello_sub: Sender<()>,
    _watcher: JoinHandle<()>,
}

impl WsContext {
    pub fn new() -> WsContext {
        let (tx, rx) = channel::<()>(BROADCAST_CHANNEL_CAPACITY);

        let t2 = tx.clone();

        let _watcher = tokio::task::spawn(async move {
            loop {
                // TODO: monitor/dispatch all the things.
                tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
                if let Err(e) = t2.send(()) {
                    info!("No subscribers present {:?}", e);
                }
            }
        });

        WsContext {
            hello_sub: tx,
            _watcher,
        }
    }
}

pub fn init_methods(database_url: &str) -> RpcModule<WsContext> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");

    let mut conn = pool.get().expect("Could not get pooled connection!");

    migrations::run_migrations(&mut *conn).expect("Failed to run database migrations!");

    let mut module = RpcModule::new(WsContext::new());

    module
        .register_subscription(
            "subscribe_hello",
            "s_hello",
            "unsubscribe_hello",
            |params, sink, ctx| methods::spawn_hello(params, sink, ctx),
        )
        .unwrap();

    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpsee::{
        core::{
            client::ClientT,
            params::{ArrayParams, ObjectParams},
        },
        http_client::HttpClientBuilder,
        server::ServerBuilder,
    };

    #[tokio::test]
    async fn test_hello() {
        let mut server = ServerBuilder::new()
            .build("0.0.0.0:8979")
            .await
            .expect("Builder failed");

        let handle = server
            .start(init_methods())
            .expect("Server failed to start");

        let client = HttpClientBuilder::default()
            .build("http://127.0.0.1:8979")
            .expect("Client builder failed");

        let response: String = client
            .request("hello_method", ObjectParams::new())
            .await
            .expect("Client call failed");

        assert_eq!("Hello, world!".to_string(), response);
    }
}
