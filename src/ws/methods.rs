use crate::database::*;
use futures_util::StreamExt;
use jsonrpsee::{
    core::error::Error,
    server::{logger::Params, SubscriptionSink},
    types::error::SubscriptionResult,
};
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use super::WsContext;

pub(super) fn spawn_live_price_data(
    params: Params<'_>,
    mut sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    let rx = ctx.price_feed.subscribe();
    let mut stream = BroadcastStream::new(rx);
    sink.accept()?;

    tokio::task::spawn(async move {
        while let Some(mesg) = stream.next().await {

            match mesg {
                Ok(m) => {
                    if let Err(e) = sink.send(&m) {
                        error!("Could not send price data to subscriber! {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Price feed stream is broken.");
                    break;
                }
            }

            if sink.is_closed() {
                break;
            }
        }
    });

    Ok(())
}
