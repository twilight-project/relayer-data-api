use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use jsonrpsee::{core::error::Error, server::logger::Params, RpcModule};
use serde::Serialize;

mod methods;
mod types;
pub use types::{
    CandleSubscription, Candles, HistoricalFundingArgs, HistoricalPriceArgs, Interval,
    OrderHistoryArgs, OrderId, PnlArgs, TradeVolumeArgs,
};

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

type HandlerType<R> =
    Box<dyn 'static + Fn(Params<'_>, &RelayerContext) -> Result<R, Error> + Send + Sync>;

pub struct RelayerContext {
    pub pool: ManagedPool,
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

pub fn init_methods(database_url: &str) -> RpcModule<RelayerContext> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");

    let mut module = RpcModule::new(RelayerContext { pool });

    register_method(
        &mut module,
        "unrealized_pnl",
        Box::new(methods::unrealized_pnl),
    );
    register_method(&mut module, "open_orders", Box::new(methods::open_orders));
    register_method(
        &mut module,
        "order_history",
        Box::new(methods::order_history),
    );
    register_method(&mut module, "trade_volume", Box::new(methods::trade_volume));
    register_method(
        &mut module,
        "get_funding_payment",
        Box::new(methods::get_funding_payment),
    );
    register_method(
        &mut module,
        "last_order_detail",
        Box::new(methods::last_order_detail),
    );
    register_method(
        &mut module,
        "lend_pool_info",
        Box::new(methods::lend_pool_info),
    );
    register_method(
        &mut module,
        "trader_order_info",
        Box::new(methods::trader_order_info),
    );
    register_method(
        &mut module,
        "lend_order_info",
        Box::new(methods::lend_order_info),
    );
    register_method(
        &mut module,
        "get_funding_rate",
        Box::new(methods::get_funding_rate),
    );
    register_method(
        &mut module,
        "btc_usd_price",
        Box::new(methods::btc_usd_price),
    );
    register_method(
        &mut module,
        "historical_price",
        Box::new(methods::historical_price),
    );
    register_method(
        &mut module,
        "historical_funding_rate",
        Box::new(methods::historical_funding_rate),
    );
    register_method(
        &mut module,
        "open_limit_orders",
        Box::new(methods::open_limit_orders),
    );
    register_method(
        &mut module,
        "recent_trade_orders",
        Box::new(methods::recent_trade_orders),
    );
    register_method(&mut module, "candle_data", Box::new(methods::candle_data));
    register_method(&mut module, "server_time", Box::new(methods::server_time));
    register_method(
        &mut module,
        "position_size",
        Box::new(methods::position_size),
    );
    // TODO:
    //register_method(
    //    &mut module,
    //    "last_day_apy",
    //    Box::new(methods::last_day_apy),
    //);

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
