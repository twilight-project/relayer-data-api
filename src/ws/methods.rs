#![allow(warnings)]
use crate::{
    database::{Ask, Bid, BtcUsdPrice, OrderBook, TraderOrder},
    error::ApiError,
    rpc::{order_book, CandleSubscription, Interval},
};
use chrono::prelude::*;
use jsonrpsee::{
    server::{logger::Params, SubscriptionSink},
    types::error::SubscriptionResult,
};
use log::{error, info, warn};
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::broadcast::{channel, error::TryRecvError, Receiver},
    task::JoinHandle,
    time::sleep,
};

use super::WsContext;

fn pipe<T>(task_name: String, mut rx: Receiver<T>, mut sink: SubscriptionSink) -> JoinHandle<()>
where
    T: Clone + Serialize + std::marker::Send + 'static,
{
    tokio::task::spawn(async move {
        loop {
            match rx.try_recv() {
                Ok(mesg) => {
                    if let Err(e) = sink.send(&mesg) {
                        error!("{}: Could not send data to subscriber! {:?}", task_name, e);
                        break;
                    }
                }
                Err(TryRecvError::Closed) => {
                    info!("{}: Channel closed", task_name);
                    break;
                }
                Err(TryRecvError::Lagged(by)) => {
                    warn!("{}: Channel is lagging by {} messages", task_name, by);
                }
                Err(TryRecvError::Empty) => {
                    sleep(Duration::from_millis(100)).await;
                }
            }

            if sink.is_closed() {
                info!("{}: subscriber closed, exiting.", task_name);
                break;
            }
        }
    })
}

pub(super) fn candle_update(
    params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    sink.accept()?;

    let CandleSubscription { interval } = params.parse()?;

    let spawn = match ctx.candles.read() {
        Ok(r) => !r.contains_key(&interval),
        Err(e) => {
            sink.send(&format!("RwLock poisoned!"));
            return Ok(());
        }
    };

    if spawn {
        info!("SPAWNING new subscriber for {:?}", interval);
        let Ok(mut l) = ctx.candles.write() else {
            sink.send(&"Write Lock poisoned!");
            return Ok(());
        };
        let (tx, _) = channel(10);
        l.insert(interval, tx.clone());

        let c = ctx.clone();
        let _: JoinHandle<Result<(), ApiError>> = tokio::task::spawn(async move {
            loop {
                let mut conn = c.pool.get()?;
                let since: DateTime<Utc> = match interval {
                    Interval::ONE_DAY_CHANGE => Utc::now() - chrono::Duration::hours(24),
                    _ => Utc::now() - chrono::Duration::milliseconds(250),
                };
                let candles = BtcUsdPrice::candles(&mut conn, interval, since, None, None)?;

                if candles.len() > 0 {
                    let result = serde_json::to_value(&candles)?;
                    if let Err(e) = tx.send(result) {
                        error!("Error sending candle updates: {:?}", e);
                    }
                    match interval {
                        Interval::ONE_DAY_CHANGE => {
                            sleep(Duration::from_millis(1000)).await;
                        }
                        _ => {
                            sleep(Duration::from_millis(250)).await;
                        }
                    };
                } else {
                    sleep(Duration::from_millis(250)).await;
                }
            }
            Ok(())
        });
    }

    let Ok(l) = ctx.candles.read() else {
        sink.send(&"Failed to acquire rx candles channel");
        return Ok(());
    };

    let mut rx = l.get(&interval).unwrap().subscribe();
    let _result = tokio::task::spawn(async move {
        loop {
            let Ok(msg) = rx.recv().await else {
                error!("Recv channel broken!");
                break;
            };

            if let Err(e) = sink.send(&msg) {
                error!("Error sending candle updates: {:?}", e);
            }
        }
        // Ok(())
    });

    Ok(())
}

pub(super) fn spawn_order_book(
    _params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    let mut rx = ctx.order_book.subscribe();
    sink.accept()?;

    let _: JoinHandle<Result<(), ApiError>> = tokio::task::spawn(async move {
        loop {
            match rx.try_recv() {
                Ok(mesg) => {
                    let mut conn = ctx.pool.get()?;
                    let mut redis_conn = ctx.client.get_connection().expect("REDIS connection.");
                    let mut orders = order_book(&mut redis_conn);
                    let result = serde_json::to_value(&orders.add_order(mesg))?;

                    if let Err(e) = sink.send(&result) {
                        error!("Error sending orderbook updates: {:?}", e);
                    }
                    sleep(Duration::from_secs(5)).await;
                }
                Err(TryRecvError::Closed) => {
                    info!("order_book: Channel closed");
                    break;
                }
                Err(TryRecvError::Lagged(by)) => {
                    warn!("order_book: Channel is lagging by {} messages", by);
                }
                Err(TryRecvError::Empty) => {
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
        Ok(())
    });

    Ok(())
}

pub(super) fn heartbeat(
    _params: Params<'_>,
    mut sink: SubscriptionSink,
    _ctx: Arc<WsContext>,
) -> SubscriptionResult {
    sink.accept()?;

    let _: JoinHandle<Result<(), ApiError>> = tokio::task::spawn(async move {
        loop {
            let result = serde_json::to_value(&"BEAT")?;
            if let Err(e) = sink.send(&result) {
                error!("Error sending hearbeat: {:?}", e);
            }
            sleep(Duration::from_secs(5)).await;
        }
        // Ok(())
    });

    Ok(())
}

pub(super) fn recent_trades(
    _params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    let rx = ctx.recent_trades.subscribe();
    sink.accept()?;

    let _ = pipe("Recent Trades".into(), rx, sink);

    Ok(())
}

pub(super) fn spawn_live_price_data(
    _params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    let rx = ctx.price_feed.subscribe();
    sink.accept()?;

    let _ = pipe("Live Price Feed".into(), rx, sink);

    Ok(())
}
