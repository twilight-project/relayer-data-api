use crate::database::*;
use futures_util::StreamExt;
use jsonrpsee::{
    core::error::Error,
    server::{logger::Params, SubscriptionSink},
    types::error::SubscriptionResult,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::broadcast::{error::TryRecvError, Receiver},
    task::JoinHandle,
    time::sleep,
};
use uuid::Uuid;

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

pub(super) fn spawn_order_book(
    params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    let mut rx = ctx.order_book.subscribe();
    sink.accept()?;

    let _ = pipe("Order Book".into(), rx, sink);

    Ok(())
}

pub(super) fn spawn_live_price_data(
    params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    let mut rx = ctx.price_feed.subscribe();
    sink.accept()?;

    let _ = pipe("Live Price Feed".into(), rx, sink);

    Ok(())
}
