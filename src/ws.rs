use crate::database::{NewOrderBookOrder, TraderOrder};
use crate::kafka::start_consumer;
use crate::rpc::Interval;
// use bigdecimal::ToPrimitive;
use chrono::prelude::*;
use crossbeam_channel::{unbounded, Sender as CrossbeamSender};
use diesel::prelude::PgConnection;
use diesel::r2d2::ConnectionManager;
use jsonrpsee::RpcModule;
use log::{error, info, trace};
use redis::Client;
use relayer_core::db::Event;
use relayer_core::relayer::PositionType;
use relayer_core::twilight_relayer_sdk::twilight_client_sdk::relayer_types::{
    OrderStatus, OrderType,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::RwLock,
    time::{Duration, Instant},
};
use tokio::{
    sync::broadcast::{channel, Sender},
    task::JoinHandle,
};

mod methods;

// const SNAPSHOT_TOPIC: &str = "CoreEventLogTopic";
// const WEBSOCKET_GROUP: &str = "Websocket";
const WS_UPDATE_INTERVAL: u64 = 250;

const BROADCAST_CHANNEL_CAPACITY: usize = 20;

type ManagedConnection = ConnectionManager<PgConnection>;
type ManagedPool = r2d2::Pool<ManagedConnection>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct RecentOrder {
    order_id: String,
    side: PositionType,
    price: f64,
    positionsize: f64,
    timestamp: String,
}
pub struct WsContext {
    client: Client,
    price_feed: Sender<(f64, DateTime<Utc>)>,
    order_book: Sender<NewOrderBookOrder>,
    recent_trades: Sender<RecentOrder>,
    pub candles: RwLock<HashMap<Interval, Sender<serde_json::Value>>>,
    pub pool: ManagedPool,
    _completions: CrossbeamSender<crate::kafka::Completion>,
    _watcher: JoinHandle<()>,
    _kafka_sub: std::thread::JoinHandle<()>,
}

impl WsContext {
    pub fn with_pool(pool: ManagedPool, client: Client) -> WsContext {
        dotenv::dotenv().ok();
        let snapshot_topic =
            std::env::var("CORE_EVENT_LOG").unwrap_or("CoreEventLogTopic".to_string());
        let websocket_group =
            std::env::var("WEBSOCKET_KAFKA_GROUP").unwrap_or("Websocket".to_string());
        let (price_feed, _) = channel::<(f64, DateTime<Utc>)>(BROADCAST_CHANNEL_CAPACITY);
        let (order_book, _) = channel::<NewOrderBookOrder>(BROADCAST_CHANNEL_CAPACITY);
        let (recent_trades, _) = channel::<RecentOrder>(BROADCAST_CHANNEL_CAPACITY);

        let price_feed2 = price_feed.clone();
        let order_book2 = order_book.clone();
        let recent_trades2 = recent_trades.clone();

        let (completions, rx, _kafka_sub) = {
            let (tx, rx) = unbounded();
            let (completions, h) = start_consumer(websocket_group, snapshot_topic, tx);

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
                                Event::FeeUpdate(cmd, event_time) => match cmd {
                                    relayer_core::relayer::RelayerCommand::UpdateFees(
                                        order_filled_on_market,
                                        order_filled_on_limit,
                                        order_settled_on_limit,
                                        order_settled_on_market,
                                    ) => {
                                        info!(
                                            "Fee update: {:?}, {:?}, {:?}, {:?} at {:?}",
                                            order_filled_on_market,
                                            order_filled_on_limit,
                                            order_settled_on_limit,
                                            order_settled_on_market,
                                            event_time,
                                        );
                                    }
                                    _ => {}
                                },
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
                                        OrderStatus::SETTLED
                                        | OrderStatus::FILLED
                                        | OrderStatus::LIQUIDATE => {
                                            let recent_order = RecentOrder {
                                                order_id: to.uuid.to_string(),
                                                side: to.position_type.into(),
                                                price: to.entryprice.into(),
                                                positionsize: to.positionsize.into(),
                                                timestamp: to.timestamp,
                                            };
                                            let _ = recent_trades2.send(recent_order);
                                        }
                                        _ => {}
                                    }
                                }
                                // added for limit order update for settlement order
                                Event::TraderOrderLimitUpdate(to, cmd, _seq) => {
                                    let settlement_price = match cmd {
                                        relayer_core::relayer::RpcCommand::ExecuteTraderOrder(
                                            execute_trader_order,
                                            _meta,
                                            _zkos_hex_string,
                                            _request_id,
                                        ) => execute_trader_order.execution_price,
                                        _ => 0.0, // Default value for other command types
                                    };

                                    let order = NewOrderBookOrder::new_close_limit(
                                        TraderOrder::from(to.clone()),
                                        settlement_price,
                                    );
                                    if let Err(e) = order_book2.send(order) {
                                        info!("No order book subscribers present {:?}", e);
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
                                Event::TxHash(..) => {}
                                Event::TxHashUpdate(..) => {}
                                Event::AdvanceStateQueue(..) => {}
                                Event::RiskEngineUpdate(..) => {}
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
            client,
            price_feed,
            order_book,
            recent_trades,
            candles: Default::default(),
            pool,
            _completions: completions,
            _watcher,
            _kafka_sub,
        }
    }
}

pub fn init_methods(database_url: &str, redis_url: &str) -> RpcModule<WsContext> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(50)
        .build(manager)
        .expect("Could not instantiate connection pool");
    let client = Client::open(redis_url).expect("Could not establish redis connection");

    let mut module = RpcModule::new(WsContext::with_pool(pool, client));

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
    // use super::*;
    // use jsonrpsee::{
    //     core::{
    //         client::ClientT,
    //         params::{ArrayParams, ObjectParams},
    //     },
    //     http_client::HttpClientBuilder,
    //     server::ServerBuilder,
    // };

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
