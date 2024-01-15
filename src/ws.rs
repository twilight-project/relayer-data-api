use crate::database::{NewOrderBookOrder, TraderOrder};
use crate::kafka::start_consumer;
use bigdecimal::ToPrimitive;
use chrono::prelude::*;
use crossbeam_channel::{unbounded, Sender as CrossbeamSender};
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use jsonrpsee::RpcModule;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::{
    sync::broadcast::{channel, Sender},
    task::JoinHandle,
};
use twilight_relayer_rust::{db::Event, relayer};
use zkoswalletlib::relayer_types::{OrderType, OrderStatus};

mod methods;

const SNAPSHOT_TOPIC: &str = "CoreEventLogTopic";
const WEBSOCKET_GROUP: &str = "Websocket";
const WS_UPDATE_INTERVAL: u64 = 250;

const BROADCAST_CHANNEL_CAPACITY: usize = 20;

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

pub struct WsContext {
    price_feed: Sender<(f64, DateTime<Utc>)>,
    order_book: Sender<NewOrderBookOrder>,
    recent_trades: Sender<relayer::TraderOrder>,
    pub pool: ManagedPool,
    _completions: CrossbeamSender<crate::kafka::Completion>,
    _watcher: JoinHandle<()>,
    _kafka_sub: std::thread::JoinHandle<()>,
}

impl WsContext {
    pub fn with_pool(pool: ManagedPool) -> WsContext {
        let (price_feed, _) = channel::<(f64, DateTime<Utc>)>(BROADCAST_CHANNEL_CAPACITY);
        let (order_book, _) = channel::<NewOrderBookOrder>(BROADCAST_CHANNEL_CAPACITY);
        let (recent_trades, _) = channel::<relayer::TraderOrder>(BROADCAST_CHANNEL_CAPACITY);

        let price_feed2 = price_feed.clone();
        let order_book2 = order_book.clone();
        let recent_trades2 = recent_trades.clone();

        let (completions, rx, _kafka_sub) = {
            let (tx, rx) = unbounded();
            let (completions, h) =
                start_consumer(WEBSOCKET_GROUP.into(), SNAPSHOT_TOPIC.into(), tx);

            (completions, rx, h)
        };

        let notify = completions.clone();

        let _watcher = tokio::task::spawn(async move {
            let mut deadline = Instant::now() + Duration::from_millis(WS_UPDATE_INTERVAL);
            loop {
                match rx.recv_deadline(deadline) {
                    Ok((completion, msgs)) => {
                        for msg in msgs {
                            match msg {
                                Event::TraderOrder(to, ..)
                                | Event::TraderOrderUpdate(to, ..)
                                | Event::TraderOrderFundingUpdate(to, ..)
                                | Event::TraderOrderLiquidation(to, ..) => {
                                    if to.order_type == OrderType::LIMIT {
                                        let order =
                                            NewOrderBookOrder::new(TraderOrder::from(to.clone()));
                                        if let Err(e) = order_book2.send(order) {
                                            info!("No order book subscribers present {:?}", e);
                                        }
                                    }

                                    match to.order_status {
                                        OrderStatus::PENDING | OrderStatus::FILLED => {
                                            recent_trades2.send(to);
                                        }
                                        _ => {}
                                    }
                                }
                                Event::LendOrder(_lend_order, _cmd, _seq) => {}
                                Event::FundingRateUpdate(
                                    _funding_rate,
                                    _btc_price,
                                    _system_time,
                                ) => {}
                                Event::CurrentPriceUpdate(current_price, system_time) => {
                                    let ts = DateTime::parse_from_rfc3339(&system_time)
                                        .expect("Bad datetime format")
                                        .into();
                                    if let Err(e) = price_feed2.send((current_price, ts)) {
                                        info!("No subscribers present {:?}", e);
                                    }
                                }
                                Event::PoolUpdate(_lend_pool_command, ..) => {}
                                Event::SortedSetDBUpdate(_sorted_set_command) => {}
                                Event::PositionSizeLogDBUpdate(
                                    _position_size_log_command,
                                    _position_size_log,
                                ) => {}
                                Event::Stop(_stop) => {}
                            }
                        }
                        if let Err(e) = notify.send(completion) {
                            error!("Crossbeam channel is closed {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        if e.is_timeout() {
                            trace!("Timeout reached");
                            deadline = Instant::now() + Duration::from_millis(WS_UPDATE_INTERVAL);
                            trace!("New deadline: {:?}", deadline);
                        } else {
                            error!("Channel disconnected!");
                            break;
                        }
                    }
                }
            }
        });

        WsContext {
            price_feed,
            order_book,
            recent_trades,
            pool,
            _completions: completions,
            _watcher,
            _kafka_sub,
        }
    }
}

pub fn init_methods(database_url: &str) -> RpcModule<WsContext> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Could not instantiate connection pool");

    let mut module = RpcModule::new(WsContext::with_pool(pool));

    module
        .register_subscription(
            "subscribe_live_price_data",
            "s_live_price_data",
            "unsubscribe_live_price_data",
            methods::spawn_live_price_data,
        )
        .unwrap();

    module
        .register_subscription(
            "subscribe_order_book",
            "s_order_book",
            "unsubscribe_order_book",
            methods::spawn_order_book,
        )
        .unwrap();

    module
        .register_subscription(
            "subscribe_candle_data",
            "s_candle_data",
            "unsubscribe_candle_data",
            methods::candle_update,
        )
        .unwrap();

    module
        .register_subscription(
            "subscribe_recent_trades",
            "s_recent_trades",
            "unsubscribe_recent_trades",
            methods::recent_trades,
        )
        .unwrap();

    module
        .register_subscription(
            "subscribe_heartbeat",
            "s_heartbeat",
            "unsubscribe_heartbeat",
            methods::heartbeat,
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
