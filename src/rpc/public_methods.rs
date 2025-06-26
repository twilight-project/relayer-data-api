use super::*;
use crate::database::*;
use chrono::prelude::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use kafka::producer::Record;
use relayerwalletlib::verify_client_message::{
    verify_client_create_trader_order, verify_query_order, verify_settle_requests,
    verify_trade_lend_order,
};
use twilight_relayer_rust::relayer;
use zkoswalletlib::relayer_rpcclient::method::RequestResponse;

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
    let args = match params.parse::<HistoricalPriceArgs>() {
        Ok(args) => args,
        Err(e) => return Err(Error::Custom(format!("Invalid argument: {:?}", e))),
    };

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
    let args: HistoricalFundingArgs = params.parse()?;

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
    let meta = relayer::Meta::default();
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
    let meta = relayer::Meta::default();
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

    let meta = relayer::Meta::default();

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

    let meta = relayer::Meta::default();

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

    let meta = relayer::Meta::default();

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
