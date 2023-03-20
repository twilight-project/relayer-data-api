use crate::database::*;
use jsonrpsee::{
    core::error::Error,
    server::{logger::Params, SubscriptionSink},
    types::error::SubscriptionResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::WsContext;

pub(super) fn spawn_hello(
    params: Params<'_>,
    sink: SubscriptionSink,
    ctx: Arc<WsContext>,
) -> SubscriptionResult {
    Ok(())
}
