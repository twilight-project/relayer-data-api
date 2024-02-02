use super::*;
use crate::{auth::AuthInfo, database::*};
use chrono::prelude::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use kafka::producer::Record;
use log::info;
use relayerwalletlib::verify_client_message::{
    verify_query_order, verify_settle_requests, verify_trade_lend_order,
};
use twilight_relayer_rust::relayer;

pub(super) fn submit_lend_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Order> = params.parse()?;
    let (customer_id, order) = args.unpack();

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

    order.account_id = tx.input.input.as_owner_address().cloned().unwrap();
    let deposit = order.deposit / 10000.0;
    let balance = order.balance / 10000.0;
    order.deposit = deposit;
    order.balance = balance;

    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    if let Err(_) = AddressCustomerId::insert(&mut conn, customer_id, &order.account_id) {
        return Ok(format!("Failed to update customer id!").into());
    }

    let order =
        relayer::RpcCommand::CreateLendOrder(order.clone(), meta, tx.input.encode_as_hex_string());
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

pub(super) fn settle_lend_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Order> = params.parse()?;
    let (customer_id, order) = args.unpack();

    let Order { data } = order;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::ExecuteLendOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_settle_requests(&tx.msg) {
        return Ok(format!("Invalid order params").into());
    }

    let mut order = tx.execute_lend_order.clone();

    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    let Ok(ord) = LendOrder::get(
        &mut conn,
        customer_id,
        OrderId {
            id: order.uuid.to_string(),
        },
    ) else {
        return Ok(format!("Order not found").into());
    };

    if ord.order_status.is_closed() {
        return Ok(format!("Order closed").into());
    }

    let meta = relayer::Meta::default();

    let Some(account_id) = tx.msg.output.as_output_data().get_owner_address() else {
        return Ok(format!("Missing owner address").into());
    };

    order.account_id = account_id.to_string();

    // TODO: what goes in HexString??
    let order =
        relayer::RpcCommand::ExecuteLendOrder(order.clone(), meta, "What goes here??".into());
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "ExecuteLendOrder", serialized);
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
    let (customer_id, order) = args.unpack();

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

    order.account_id = tx.input.input.as_owner_address().cloned().unwrap();
    let margin = order.initial_margin / 10000.0;
    order.initial_margin = margin;
    order.available_margin = margin;

    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    if let Err(_) = AddressCustomerId::insert(&mut conn, customer_id, &order.account_id) {
        return Ok(format!("Failed to update customer id!").into());
    }

    let order = relayer::RpcCommand::CreateTraderOrder(
        order.clone(),
        meta,
        tx.input.encode_as_hex_string(),
    );
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

pub(super) fn settle_trade_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Order> = params.parse()?;
    let (customer_id, order) = args.unpack();

    let Order { data } = order;

    let Ok(bytes) = hex::decode(&data) else {
        return Ok(format!("Invalid hex data").into());
    };

    let Ok(tx) = bincode::deserialize::<relayer::ExecuteTraderOrderZkos>(&bytes) else {
        return Ok(format!("Invalid bincode").into());
    };

    if let Err(_) = verify_settle_requests(&tx.msg) {
        return Ok(format!("Invalid order params").into());
    }

    let mut order = tx.execute_trader_order.clone();

    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    let Ok(ord) = TraderOrder::get(&mut conn, customer_id, order.uuid.to_string()) else {
        return Ok(format!("Order not found").into());
    };

    if ord.order_status.is_closed() {
        return Ok(format!("Order closed").into());
    }

    let meta = relayer::Meta::default();

    let Some(account_id) = tx.msg.output.as_output_data().get_owner_address() else {
        return Ok(format!("Missing owner address").into());
    };

    order.account_id = account_id.to_string();

    // TODO: HexString??
    let order =
        relayer::RpcCommand::ExecuteTraderOrder(order.clone(), meta, "What here??".to_string());
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "ExecuteTraderOrder", serialized);
    if let Err(e) = ctx.kafka.lock().expect("Lock poisoned!").send(&record) {
        Ok(format!("Could not send order {:?}", e).into())
    } else {
        Ok("OK".into())
    }
}

pub(super) fn cancel_order(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Order> = params.parse()?;
    let (customer_id, order) = args.unpack();

    let Order { data } = order;

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

    let mut order = tx.cancel_trader_order.clone();

    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    let Ok(ord) = TraderOrder::get(&mut conn, customer_id, order.uuid.to_string()) else {
        return Ok(format!("Order not found").into());
    };

    if !ord.order_status.is_cancelable() {
        return Ok(format!("Order not cancelable").into());
    }

    let meta = relayer::Meta::default();

    let account_id = tx.msg.public_key.clone();
    order.account_id = account_id;

    // TODO: HexString??
    let order =
        relayer::RpcCommand::CancelTraderOrder(order.clone(), meta, "What here??".to_string());
    let Ok(serialized) = serde_json::to_vec(&order) else {
        return Ok(format!("Could not serialize order").into());
    };

    let record = Record::from_key_value(&topic, "CancelTraderOrder", serialized);
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
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::get(&mut conn, id, params.id) {
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
    let args: RpcArgs<OrderId> = params.parse()?;
    let (id, params) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match FundingRate::funding_payment(&mut conn, id, params.id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!(
                "Error fetching funding payment: {:?}",
                e
            ))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn last_order_detail(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args: RpcArgs<()> = params.parse()?;
    let (id, _) = args.unpack();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::last_order(&mut conn, id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching last order: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}
