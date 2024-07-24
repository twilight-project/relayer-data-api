use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use jsonrpsee::{core::error::Error, server::logger::Params, RpcModule};
use kafka::producer::{Producer, RequiredAcks};
use redis::Client;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

mod private_methods;
mod public_methods;
mod types;
mod util;

pub use types::{
    CandleSubscription, Candles, HistoricalFundingArgs, HistoricalPriceArgs, Interval, Order,
    OrderHistoryArgs, OrderId, PnlArgs, RpcArgs, TradeVolumeArgs, TransactionHashArgs,
};
pub use util::{order_book, recent_orders};

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

type HandlerType<R> =
    Box<dyn 'static + Fn(Params<'_>, &RelayerContext) -> Result<R, Error> + Send + Sync>;

pub struct RelayerContext {
    pub pool: ManagedPool,
    pub client: Client,
    pub kafka: Arc<Mutex<Producer>>,
}

fn register_method<R: Serialize + 'static>(
    module: &mut RpcModule<RelayerContext>,
    name: &'static str,
    method: HandlerType<R>,
) {
    if let Err(e) = module.register_method(name, method) {
        panic!("API failed to register {}! {:?}", name, e);
    }
}

pub fn init_public_methods(database_url: &str, redis_url: &str) -> RpcModule<RelayerContext> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");
    let client = Client::open(redis_url).expect("Could not establish redis connection");

    let broker_host = std::env::var("BROKER").expect("missing environment variable BROKER");
    let broker = vec![broker_host.to_owned()];
    let kafka = Producer::from_hosts(broker)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();
    let kafka = Arc::new(Mutex::new(kafka));

    let mut module = RpcModule::new(RelayerContext {
        client,
        pool,
        kafka,
    });
    register_method(
        &mut module,
        "btc_usd_price",
        Box::new(public_methods::btc_usd_price),
    );
    register_method(
        &mut module,
        "historical_price",
        Box::new(public_methods::historical_price),
    );
    register_method(
        &mut module,
        "candle_data",
        Box::new(public_methods::candle_data),
    );
    register_method(
        &mut module,
        "server_time",
        Box::new(public_methods::server_time),
    );
    register_method(
        &mut module,
        "get_funding_rate",
        Box::new(public_methods::get_funding_rate),
    );
    register_method(
        &mut module,
        "historical_funding_rate",
        Box::new(public_methods::historical_funding_rate),
    );
    register_method(
        &mut module,
        "open_limit_orders",
        Box::new(public_methods::open_limit_orders),
    );
    register_method(
        &mut module,
        "recent_trade_orders",
        Box::new(public_methods::recent_trade_orders),
    );
    register_method(
        &mut module,
        "position_size",
        Box::new(public_methods::position_size),
    );
    register_method(
        &mut module,
        "transaction_hashes",
        Box::new(public_methods::transaction_hashes),
    );
    register_method(
        &mut module,
        "trader_order_info",
        Box::new(public_methods::trader_order_info),
    );
    register_method(
        &mut module,
        "lend_order_info",
        Box::new(public_methods::lend_order_info),
    );
    register_method(
        &mut module,
        "submit_trade_order",
        Box::new(public_methods::submit_trade_order),
    );
    register_method(
        &mut module,
        "submit_lend_order",
        Box::new(public_methods::submit_lend_order),
    );
    register_method(
        &mut module,
        "settle_trade_order",
        Box::new(public_methods::settle_trade_order),
    );
    register_method(
        &mut module,
        "settle_lend_order",
        Box::new(public_methods::settle_lend_order),
    );
    register_method(
        &mut module,
        "cancel_trader_order",
        Box::new(public_methods::cancel_trader_order),
    );
    module
}

pub fn init_private_methods(database_url: &str, redis_url: &str) -> RpcModule<RelayerContext> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");
    let client = Client::open(redis_url).expect("Could not establish redis connection");

    let broker_host = std::env::var("BROKER").expect("missing environment variable BROKER");
    let broker = vec![broker_host.to_owned()];
    let kafka = Producer::from_hosts(broker)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();
    let kafka = Arc::new(Mutex::new(kafka));

    let mut module = RpcModule::new(RelayerContext {
        client,
        pool,
        kafka,
    });

    register_method(
        &mut module,
        "submit_lend_order",
        Box::new(private_methods::submit_lend_order),
    );
    register_method(
        &mut module,
        "settle_lend_order",
        Box::new(private_methods::settle_lend_order),
    );
    register_method(
        &mut module,
        "submit_trade_order",
        Box::new(private_methods::submit_trade_order),
    );
    register_method(
        &mut module,
        "settle_trade_order",
        Box::new(private_methods::settle_trade_order),
    );
    register_method(
        &mut module,
        "cancel_trader_order",
        Box::new(private_methods::cancel_trader_order),
    );
    register_method(
        &mut module,
        "submit_bulk_order",
        Box::new(private_methods::submit_bulk_order),
    );
    register_method(
        &mut module,
        "unrealized_pnl",
        Box::new(private_methods::unrealized_pnl),
    );
    register_method(
        &mut module,
        "open_orders",
        Box::new(private_methods::open_orders),
    );
    register_method(
        &mut module,
        "order_history",
        Box::new(private_methods::order_history),
    );
    register_method(
        &mut module,
        "trade_volume",
        Box::new(private_methods::trade_volume),
    );
    register_method(
        &mut module,
        "get_funding_payment",
        Box::new(private_methods::get_funding_payment),
    );
    register_method(
        &mut module,
        "last_order_detail",
        Box::new(private_methods::last_order_detail),
    );
    register_method(
        &mut module,
        "lend_pool_info",
        Box::new(private_methods::lend_pool_info),
    );
    register_method(
        &mut module,
        "trader_order_info",
        Box::new(private_methods::trader_order_info),
    );
    register_method(
        &mut module,
        "lend_order_info",
        Box::new(private_methods::lend_order_info),
    );
    // TODO:
    //register_method(
    //    &mut module,
    //    "last_day_apy",
    //    Box::new(private_methods::last_day_apy),
    //);

    module
}

#[cfg(test)]
mod tests {
    use crate::ws::init_methods;

    use super::*;
    use jsonrpsee::{
        core::{
            client::ClientT,
            params::{ArrayParams, ObjectParams},
        },
        http_client::HttpClientBuilder,
        server::ServerBuilder,
    };

    // #[tokio::test]
    // async fn test_hello() {
    //     let mut server = ServerBuilder::new()
    //         .build("0.0.0.0:8979")
    //         .await
    //         .expect("Builder failed");

    //     let handle = server
    //         .start(init_methods())
    //         .expect("Server failed to start");

    //     let client = HttpClientBuilder::default()
    //         .build("http://127.0.0.1:8979")
    //         .expect("Client builder failed");

    //     let response: String = client
    //         .request("hello_method", ObjectParams::new())
    //         .await
    //         .expect("Client call failed");

    //     assert_eq!("Hello, world!".to_string(), response);
    // }
}
