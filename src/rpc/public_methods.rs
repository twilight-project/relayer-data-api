use super::*;
use super::types::RiskParams;
use bigdecimal::BigDecimal;
use crate::database::*;
use chrono::prelude::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use kafka::producer::Record;
use relayer_core::relayer;
use relayer_core::relayer::RiskState;
use relayer_core::twilight_relayer_sdk::twilight_client_sdk::relayer_rpcclient::method::RequestResponse;
use relayer_core::twilight_relayer_sdk::verify_client_message::{
    verify_client_create_trader_order, verify_query_order, verify_settle_requests,
    verify_trade_lend_order,
};

pub(super) fn btc_usd_price(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match BtcUsdPrice::get(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn historical_price(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let mut args = match params.parse::<HistoricalPriceArgs>() {
        Ok(args) => args,
        Err(e) => return Err(Error::Custom(format!("Invalid argument: {:?}", e))),
    };
    args.limit = args.limit.clamp(1, super::types::MAX_HISTORICAL_LIMIT);

    match ctx.pool.get() {
        Ok(mut conn) => match BtcUsdPrice::get_historical(&mut conn, args) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn candle_data(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Candles {
        interval,
        since,
        limit,
        offset,
    } = params.parse()?;
    let limit = limit.clamp(1, super::types::MAX_HISTORICAL_LIMIT);

    match ctx.pool.get() {
        Ok(mut conn) => {
            match BtcUsdPrice::candles(&mut conn, interval, since, Some(limit), Some(offset)) {
                Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
                Err(e) => Err(Error::Custom(format!(
                    "Error fetching candles info: {:?}",
                    e
                ))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn historical_funding_rate(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let mut args: HistoricalFundingArgs = params.parse()?;
    args.limit = args.limit.clamp(1, super::types::MAX_HISTORICAL_LIMIT);

    match ctx.pool.get() {
        Ok(mut conn) => match FundingRate::get_historical(&mut conn, args) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn get_funding_rate(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match FundingRate::get(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}
pub(super) fn historical_fee_rate(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let mut args: HistoricalFeeArgs = params.parse()?;
    args.limit = args.limit.clamp(1, super::types::MAX_HISTORICAL_LIMIT);

    match ctx.pool.get() {
        Ok(mut conn) => match FeeHistory::get_historical(&mut conn, args) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn get_fee_rate(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match FeeHistory::get(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn open_limit_orders(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Ok(mut conn) = ctx.client.get_connection() else {
        return Ok("Redis connection error.".into());
    };

    let book = order_book(&mut conn);

    Ok(serde_json::to_value(book).expect("Failed to serialize order book"))
}

pub(super) fn recent_trade_orders(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Ok(mut conn) = ctx.client.get_connection() else {
        return Ok("Redis connection error.".into());
    };

    let orders = recent_orders(&mut conn);

    Ok(serde_json::to_value(&orders).expect("Failed to serialize recent orders"))
}

pub(super) fn position_size(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match PositionSizeLog::get_latest(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!(
                "Error fetching position size: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn transaction_hashes(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: TransactionHashArgs = params.parse()?;

    match ctx.pool.get() {
        Ok(mut conn) => match TxHash::get(&mut conn, args) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!(
                "Error fetching transaction hashes: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn server_time(_: Params<'_>, _: &RelayerContext) -> Result<serde_json::Value, Error> {
    Ok(serde_json::to_value(Utc::now()).expect("Failed to get timestamp"))
}

pub(super) fn trader_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Order { data } = params.parse()?;
    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };
    // println!("bytes:{:?}", bytes);
    let Ok(tx) = bincode::deserialize::<relayer::QueryTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };
    if let Err(arg) = verify_query_order(
        tx.msg.clone(),
        &bincode::serialize(&tx.query_trader_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params:{:?}", arg).into());
    }
    match ctx.pool.get() {
        Ok(mut conn) => {
            match TraderOrder::get_by_signature(&mut conn, tx.query_trader_order.account_id) {
                Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
                Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

#[derive(Serialize)]
struct TraderOrderInfoV1 {
    #[serde(flatten)]
    order: TraderOrder,
    settle_limit: Option<SettleLimitDetails>,
    funding_applied: Option<BigDecimal>,
}

pub(super) fn trader_order_info_v1(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Order { data } = params.parse()?;
    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };
    let Ok(tx) = bincode::deserialize::<relayer::QueryTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };
    if let Err(arg) = verify_query_order(
        tx.msg.clone(),
        &bincode::serialize(&tx.query_trader_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params:{:?}", arg).into());
    }
    match ctx.pool.get() {
        Ok(mut conn) => {
            match TraderOrder::get_by_signature(&mut conn, tx.query_trader_order.account_id) {
                Ok(order) => {
                    let settle_limit = match order.order_status {
                        OrderStatus::SETTLED | OrderStatus::LIQUIDATE => None,
                        _ => SortedSetCommand::get_latest_close_limit(&mut conn, &order.uuid)
                            .unwrap_or(None),
                    };
                    let funding_applied = TraderOrderFundingUpdates::get_latest_by_uuid(
                        &mut conn,
                        &order.uuid,
                    )
                    .unwrap_or(None)
                    .map(|f| f.initial_margin - f.available_margin - f.fee_filled);
                    let response = TraderOrderInfoV1 { order, settle_limit, funding_applied };
                    Ok(serde_json::to_value(response).expect("Error converting response"))
                }
                Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

#[derive(Serialize)]
struct OrderFundingHistoryEntry {
    time: DateTime<Utc>,
    position_side: PositionType,
    payment: BigDecimal,
    funding_rate: BigDecimal,
    order_id: String,
}

pub(super) fn order_funding_history(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Order { data } = params.parse()?;
    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };
    let Ok(tx) = bincode::deserialize::<relayer::QueryTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };
    if let Err(arg) = verify_query_order(
        tx.msg.clone(),
        &bincode::serialize(&tx.query_trader_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params:{:?}", arg).into());
    }
    match ctx.pool.get() {
        Ok(mut conn) => {
            match TraderOrder::get_by_signature(&mut conn, tx.query_trader_order.account_id) {
                Ok(order) => {
                    let funding_updates = TraderOrderFundingUpdates::get_all_by_uuid(
                        &mut conn,
                        &order.uuid,
                    )
                    .unwrap_or_default();

                    let mut prev_total = BigDecimal::from(0);
                    let entries: Vec<OrderFundingHistoryEntry> = funding_updates
                        .into_iter()
                        .map(|update| {
                            let total =
                                &update.initial_margin - &update.available_margin - &update.fee_filled;
                            let payment = &total - &prev_total;
                            prev_total = total;

                            let fr = FundingRate::get_closest_before(&mut conn, update.timestamp);
                            let rate = fr.ok().flatten().map(|f| f.rate).unwrap_or_default();

                            OrderFundingHistoryEntry {
                                time: update.timestamp,
                                position_side: update.position_type,
                                payment,
                                funding_rate: rate,
                                order_id: update.uuid,
                            }
                        })
                        .collect();

                    Ok(serde_json::to_value(entries).expect("Error converting response"))
                }
                Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn lend_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Order { data } = params.parse()?;
    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };
    // println!("bytes:{:?}", bytes);
    let Ok(tx) = bincode::deserialize::<relayer::QueryLendOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };
    if let Err(arg) = verify_query_order(
        tx.msg.clone(),
        &bincode::serialize(&tx.query_lend_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params:{:?}", arg).into());
    }
    match ctx.pool.get() {
        Ok(mut conn) => {
            match LendOrder::get_by_signature(&mut conn, tx.query_lend_order.account_id) {
                Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
                Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}
pub(super) fn historical_trader_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Order { data } = params.parse()?;
    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };
    // println!("bytes:{:?}", bytes);
    let Ok(tx) = bincode::deserialize::<relayer::QueryTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };
    if let Err(arg) = verify_query_order(
        tx.msg.clone(),
        &bincode::serialize(&tx.query_trader_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params:{:?}", arg).into());
    }
    match ctx.pool.get() {
        Ok(mut conn) => {
            match TraderOrder::historical_get_by_signature(
                &mut conn,
                tx.query_trader_order.account_id,
            ) {
                Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
                Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn historical_lend_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let Order { data } = params.parse()?;
    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };
    // println!("bytes:{:?}", bytes);
    let Ok(tx) = bincode::deserialize::<relayer::QueryLendOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };
    if let Err(arg) = verify_query_order(
        tx.msg.clone(),
        &bincode::serialize(&tx.query_lend_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params:{:?}", arg).into());
    }
    match ctx.pool.get() {
        Ok(mut conn) => {
            match LendOrder::historical_get_by_signature(&mut conn, tx.query_lend_order.account_id)
            {
                Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
                Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
            }
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn submit_trade_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let Order { data } = params.parse()?;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::CreateTraderOrderClientZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_client_create_trader_order(&tx.tx) {
        return Ok(format!("Invalid order params").into());
    }

    let Ok(transaction_ser) = bincode::serialize(&tx.tx) else {
        return Ok(format!("Invalid bincode").into());
    };

    let mut order = tx.create_trader_order.clone();
    let meta = super::headers::meta_from_headers();
    let public_key = order.account_id.clone();
    order.available_margin = order.initial_margin;
    let response = RequestResponse::new(
        "Order request submitted successfully".to_string(),
        public_key,
    );
    let Ok(response_value) = serde_json::to_value(&response) else {
        return Ok(format!("Invalid response").into());
    };

    let order = relayer::RpcCommand::CreateTraderOrder(
        order.clone(),
        meta,
        hex::encode(transaction_ser),
        response.get_id(),
    );
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "CreateTraderOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok(response_value)
    }
}

pub(super) fn submit_lend_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let Order { data } = params.parse()?;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::CreateLendOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_trade_lend_order(&tx.input) {
        return Ok(format!("Invalid order params").into());
    }

    let mut order = tx.create_lend_order.clone();
    let public_key = order.account_id.clone();
    let meta = super::headers::meta_from_headers();
    order.balance = order.deposit;
    let response = RequestResponse::new(
        "Order request submitted successfully".to_string(),
        public_key,
    );
    let Ok(response_value) = serde_json::to_value(&response) else {
        return Ok(format!("Invalid response").into());
    };
    let order = relayer::RpcCommand::CreateLendOrder(
        order.clone(),
        meta,
        tx.input.encode_as_hex_string(),
        response.get_id(),
    );
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "CreateLendOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok(response_value)
    }
}

pub(super) fn settle_trade_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let Order { data } = params.parse()?;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::ExecuteTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_settle_requests(&tx.msg) {
        return Ok(format!("Invalid order params").into());
    }

    let order = tx.execute_trader_order.clone();
    let public_key = order.account_id.clone();
    let response = RequestResponse::new(
        "Order request submitted successfully".to_string(),
        public_key,
    );
    let Ok(response_value) = serde_json::to_value(&response) else {
        return Ok(format!("Invalid response").into());
    };
    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    let Ok(ord) = TraderOrder::get_by_uuid(&mut conn, order.uuid.to_string()) else {
        return Ok(format!("Order not found").into());
    };

    if !ord.order_status.is_closed() {
        return Ok(format!("Order closed").into());
    }

    let meta = super::headers::meta_from_headers();

    let order = relayer::RpcCommand::ExecuteTraderOrder(
        order.clone(),
        meta,
        tx.msg.encode_as_hex_string(),
        response.get_id(),
    );
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "ExecuteTraderOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok(response_value)
    }
}

pub(super) fn settle_lend_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let Order { data } = params.parse()?;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::ExecuteLendOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_settle_requests(&tx.msg) {
        return Ok(format!("Invalid order params").into());
    }

    let order = tx.execute_lend_order.clone();
    let public_key = order.account_id.clone();
    let response = RequestResponse::new(
        "Order request submitted successfully".to_string(),
        public_key,
    );
    let Ok(response_value) = serde_json::to_value(&response) else {
        return Ok(format!("Invalid response").into());
    };
    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    let Ok(ord) = LendOrder::get_by_uuid(&mut conn, order.uuid.to_string()) else {
        return Ok(format!("Order not found").into());
    };

    if !ord.order_status.is_closed() {
        return Ok(format!("Order closed").into());
    }

    let meta = super::headers::meta_from_headers();

    let order = relayer::RpcCommand::ExecuteLendOrder(
        order.clone(),
        meta,
        tx.msg.encode_as_hex_string(),
        response.get_id(),
    );
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "ExecuteLendOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok(response_value)
    }
}

pub(super) fn cancel_trader_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let Order { data } = params.parse()?;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::CancelTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_query_order(
        tx.msg.convert_cancel_to_query(),
        &bincode::serialize(&tx.cancel_trader_order).unwrap(),
    ) {
        return Ok(format!("Invalid order params").into());
    }

    let order = tx.cancel_trader_order.clone();
    let public_key = order.account_id.clone();
    let response = RequestResponse::new(
        "Order request submitted successfully".to_string(),
        public_key,
    );
    let Ok(response_value) = serde_json::to_value(&response) else {
        return Ok(format!("Invalid response").into());
    };
    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    let Ok(ord) = TraderOrder::get_by_uuid(&mut conn, order.uuid.to_string()) else {
        return Ok(format!("Order not found").into());
    };

    if !ord.order_status.is_cancelable() {
        return Ok(format!("Order not cancelable").into());
    }

    let meta = super::headers::meta_from_headers();

    let order = relayer::RpcCommand::CancelTraderOrder(
        order.clone(),
        meta,
        tx.msg.encode_as_hex_string(),
        response.get_id(),
    );
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "CancelTraderOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok(response_value)
    }
}

pub(super) fn pool_share_value(
    _params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match LendPool::get(&mut conn) {
            Ok(o) => {
                let value = o.get_pool_share_value();
                Ok(serde_json::to_value(value).expect("Error converting response"))
            }
            Err(e) => Err(Error::Custom(format!(
                "Error fetching lend pool info: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn lend_pool_info(
    _params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match LendPool::get(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!(
                "Error fetching lend pool info: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn last_day_apy(
    _params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match PoolAnalytics::last_day_apy_now(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}
pub(super) fn apy_chart(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    // use diesel::QueryableByName;
    use diesel::RunQueryDsl; // just the trait you need for .load()
                             // use crate::database::models::PoolAnalytics; // adjust path if needed
    let args: crate::rpc::types::ApySeriesArgs = params
        .parse()
        .map_err(|e| Error::Custom(format!("Invalid argument: {:?}", e)))?;

    let (window, step, lookback) = match args.resolve() {
        Ok(t) => t,
        Err(msg) => return Err(Error::Custom(msg)),
    };

    match ctx.pool.get() {
        Ok(mut conn) => {
            // We exposed `apy_series(window, step, lookback)` via SQL. Our Rust helper took a fixed '24 hours',
            // so we’ll call the SQL directly here to honor custom lookback as well.
            let sql = r#"
                SELECT bucket_ts, apy
                FROM apy_series($1::interval, $2::interval, $3::interval)
                WHERE apy IS NOT NULL
                ORDER BY bucket_ts;
            "#;
            let rows: Vec<ApyPoint> = diesel::sql_query(sql)
                .bind::<diesel::sql_types::Text, _>(window)
                .bind::<diesel::sql_types::Text, _>(step)
                .bind::<diesel::sql_types::Text, _>(lookback)
                .load(&mut conn)
                .map_err(|e| Error::Custom(format!("Database error: {:?}", e)))?;

            Ok(serde_json::to_value(rows).expect("Error converting response"))
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn open_interest(
    _params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match get_open_interest(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}
// pub(super) fn account_summary_by_twilight_address(
//     params: Params<'_>,
//     ctx: &RelayerContext,
// ) -> Result<serde_json::Value, Error> {
//     let args: crate::rpc::types::AccountSummaryByTAddressArgs = params.parse()?;
//     let (t_address, from, to) = args.normalize().map_err(Error::Custom)?;
//     match ctx.pool.get() {
//         Ok(mut conn) => {
//             match account_summary_by_twilight_address_fn(&mut conn, &t_address, from, to) {
//                 Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
//                 Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
//             }
//         }
//         Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
//     }
// }
pub(super) fn account_summary_by_twilight_address(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: crate::rpc::types::AccountSummaryByTAddressArgs = params.parse()?;

    let (t_address, from, to) = args.normalize().map_err(Error::Custom)?;

    match ctx.pool.get() {
        Ok(mut conn) => {
            let summary = account_summary_by_twilight_address_fn(&mut conn, &t_address, from, to)
                .map_err(|e| Error::Custom(format!("Database error: {:?}", e)))?;

            let response = crate::rpc::types::AccountSummaryByTAddressResponse {
                from,
                to,
                settled_positionsize: summary.settled_positionsize,
                filled_positionsize: summary.filled_positionsize,
                liquidated_positionsize: summary.liquidated_positionsize,
                settled_count: summary.settled_count,
                filled_count: summary.filled_count,
                liquidated_count: summary.liquidated_count,
            };

            Ok(serde_json::to_value(response).expect("Error converting response"))
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn all_account_summaries(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: crate::rpc::types::AllAccountSummariesArgs = params.parse()?;

    let (from, to, limit, offset) = args.normalize().map_err(Error::Custom)?;

    match ctx.pool.get() {
        Ok(mut conn) => {
            let rows = all_account_summaries_fn(&mut conn, from, to, limit, offset)
                .map_err(|e| Error::Custom(format!("Database error: {:?}", e)))?;

            let summaries = rows
                .into_iter()
                .map(|r| crate::rpc::types::AddressSummaryItem {
                    twilight_address: r.twilight_address,
                    settled_positionsize: r.settled_positionsize,
                    filled_positionsize: r.filled_positionsize,
                    liquidated_positionsize: r.liquidated_positionsize,
                    settled_count: r.settled_count,
                    filled_count: r.filled_count,
                    liquidated_count: r.liquidated_count,
                })
                .collect();

            let response = crate::rpc::types::AllAccountSummariesResponse {
                from,
                to,
                limit,
                offset,
                summaries,
            };

            Ok(serde_json::to_value(response).expect("Error converting response"))
        }
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn get_market_stats(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    // 1. Read latest RiskState from Redis
    let mut redis_conn = ctx
        .client
        .get_connection()
        .map_err(|e| Error::Custom(format!("Redis connection error: {:?}", e)))?;

    let state_json: Option<String> = redis::cmd("GET")
        .arg("risk_state")
        .query(&mut redis_conn)
        .unwrap_or(None);

    let risk_state: RiskState = match state_json {
        Some(json) => serde_json::from_str(&json).unwrap_or_else(|_| RiskState::new()),
        None => RiskState::new(),
    };

    // 1b. Read latest RiskParams from Redis
    let params_json: Option<String> = redis::cmd("GET")
        .arg("risk_params")
        .query(&mut redis_conn)
        .unwrap_or(None);

    let risk_params: RiskParams = match params_json {
        Some(json) => serde_json::from_str(&json).unwrap_or_else(|_| RiskParams::from_env()),
        None => RiskParams::from_env(),
    };

    // 2. Get pool equity from lend_pool table
    let mut db_conn = ctx
        .pool
        .get()
        .map_err(|e| Error::Custom(format!("Database error: {:?}", e)))?;

    let pool_equity_btc = match LendPool::get(&mut db_conn) {
        Ok(pool) => pool.get_total_locked_value(),
        Err(_) => 0.0,
    };

    // 3. Compute and return market risk stats
    let stats = util::compute_market_risk_stats(&risk_state, pool_equity_btc, risk_params);

    Ok(serde_json::to_value(stats).expect("Error converting response"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayer::QueryTraderOrderZkos;

    #[test]
    #[ignore]
    fn test_decode_trader_order_data() {
        let hex_data = "8a00000000000000306334303934663238303432653433336538373338343039303162666432626464336639613361626264656565633030633835313665363336646533313738313739386337626636333635623164616436393239623537626462336533666564376439623435356330396137623833663964623936666233306166643636393831623939373363306239050000008a00000000000000306334303934663238303432653433336538373338343039303162666432626464336639613361626264656565633030633835313665363336646533313738313739386337626636333635623164616436393239623537626462336533666564376439623435356330396137623833663964623936666233306166643636393831623939373363306239400000000000000048e6f21f906aa638efcfb5a60b6ccea44b849d6dc6500d4c43a0c4dc945ce83808d260ae4c7054f17b68421efc49e57df2126c57108fc3c5e28a9be4acb1e00c";

        // Test hex decode
        let bytes = hex::decode(hex_data).expect("Should decode hex");

        // Test bincode deserialize
        let tx: QueryTraderOrderZkos = bincode::deserialize(&bytes).expect("Should deserialize");
        println!("tx: {:?}", tx);
        // Test verify_query_order
        let verify_result = verify_query_order(
            tx.msg.clone(),
            &bincode::serialize(&tx.query_trader_order).unwrap(),
        );
        assert!(verify_result.is_ok());
    }
}
