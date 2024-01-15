use super::*;
use crate::{auth::AuthInfo, database::*};
use chrono::prelude::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use kafka::producer::Record;
use log::info;
use relayerwalletlib::verify_client_message::verify_trade_lend_order;
use twilight_relayer_rust::relayer;

pub(super) fn submit_lend_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Order> = params.parse()?;
    let (account_id, order) = args.unpack();
    let account_id = format!("{:016x}", account_id);

    let Order { data } = order;

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
    let meta = relayer::Meta::default();

    order.account_id = account_id;
    let deposit = order.deposit / 10000.0;
    let balance = order.balance / 10000.0;
    order.deposit = deposit;
    order.balance = balance;

    let order = relayer::RpcCommand::CreateLendOrder(order.clone(), meta, tx.input.encode_as_hex_string());
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "CreateLendOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok("OK".into())
    }
}

pub(super) fn submit_trade_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Order> = params.parse()?;
    let (account_id, order) = args.unpack();
    let account_id = format!("{:016x}", account_id);

    let Order { data } = order;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::CreateTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_trade_lend_order(&tx.input) {
        return Ok(format!("Invalid order params").into());
    }

    let mut order = tx.create_trader_order.clone();
    let meta = relayer::Meta::default();

    order.account_id = account_id;
    let margin = order.initial_margin / 10000.0;
    order.initial_margin = margin;
    order.available_margin = margin;

    let order = relayer::RpcCommand::CreateTraderOrder(order.clone(), meta, tx.input.encode_as_hex_string());
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "CreateTraderOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok("OK".into())
    }
}

pub(super) fn submit_bulk_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Vec<Order>> = params.parse()?;
    let (account_id, orders) = args.unpack();
    let account_id = format!("{:016x}", account_id);

    // TODO: bulk orders with ZkOS??
    Ok("OK".into())

    //let mut records = Vec::new();
    //for order in orders.into_iter() {
    //    let Order {
    //        position_type,
    //        order_type,
    //        leverage,
    //        initial_margin,
    //        available_margin,
    //        order_status,
    //        entryprice,
    //        execution_price,
    //        request_time,
    //        order_kill_time,
    //    } = order;

    //    let order = relayer::CreateTraderOrder {
    //        account_id: account_id.clone(),
    //        position_type,
    //        order_type,
    //        leverage,
    //        initial_margin,
    //        available_margin,
    //        order_status,
    //        entryprice,
    //        execution_price,
    //    };

    //    let order = relayer::RpcCommand::CreateTraderOrder(order, relayer::Meta::default());
    //    let Ok(serialized) = serde_json::to_vec(&order) else {
    //        return Ok(format!("Could not serialize order").into());
    //    };

    //    records.push(Record::from_key_value(
    //        &topic,
    //        "CreateTraderOrder",
    //        serialized,
    //    ));
    //}

    //if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send_all(&records) {
    //    Ok(format!("Could not send order {:?}", e).into())
    //} else {
    //    Ok("OK".into())
    //}
}

pub(super) fn trader_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<OrderId> = params.parse()?;

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::get(&mut conn, args.params.id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
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

pub(super) fn lend_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<OrderId> = params.parse()?;
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match LendOrder::get(&mut conn, id, params) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn last_day_apy(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    todo!("APY")
}

pub(super) fn unrealized_pnl(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<PnlArgs> = params.parse()?;
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::unrealized_pnl(&mut conn, id, params) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching pnl: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn open_orders(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<()> = params.parse()?;
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::open_orders(&mut conn, id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching pnl: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn order_history(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<OrderHistoryArgs> = params.parse()?;
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::order_history(&mut conn, id, params) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!(
                "Error fetching order history: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn trade_volume(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<TradeVolumeArgs> = params.parse()?;
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::order_volume(&mut conn, id, params) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!(
                "Error fetching order volume: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn get_funding_payment(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    unimplemented!("TODO")
}

pub(super) fn last_order_detail(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    unimplemented!("TODO")
}
