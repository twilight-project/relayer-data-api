use super::*;
use crate::database::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use kafka::producer::Record;
use relayer_core::relayer;
use relayer_core::twilight_relayer_sdk::twilight_client_sdk::relayer_rpcclient::method::RequestResponse;
use relayer_core::twilight_relayer_sdk::verify_client_message::{
    verify_client_create_trader_order, verify_query_order, verify_settle_requests,
    verify_trade_lend_order,
};

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
    let meta = super::headers::meta_from_headers();
    let public_key = order.account_id.clone();
    order.balance = order.deposit;
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

    if let Err(_) = AddressCustomerId::insert(&mut conn, customer_id, &order.account_id) {
        return Ok(format!("Failed to update customer id!").into());
    }

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

    let Ok(ord) = LendOrder::get(
        &mut conn,
        customer_id,
        OrderId {
            id: order.uuid.to_string(),
        },
    ) else {
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
    let Ok(mut conn) = ctx.pool.get() else {
        return Ok(format!("Database connection error").into());
    };

    if let Err(_) = AddressCustomerId::insert(&mut conn, customer_id, &order.account_id) {
        return Ok(format!("Failed to update customer id!").into());
    }

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

    let Ok(ord) = TraderOrder::get(&mut conn, customer_id, order.uuid.to_string()) else {
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

pub(super) fn cancel_trader_order(
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

    let Ok(ord) = TraderOrder::get(&mut conn, customer_id, order.uuid.to_string()) else {
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

pub(super) fn submit_bulk_order(
    params: Params<'_>,
    _ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let _topic = std::env::var("RPC_CLIENT_REQUEST").expect("No client topic!");
    let args: RpcArgs<Vec<Order>> = params.parse()?;
    let (account_id, _orders) = args.unpack();
    let _account_id = format!("{:016x}", account_id);

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

// pub(super) fn last_day_apy(
//     _: Params<'_>,
//     ctx: &RelayerContext,
// ) -> Result<serde_json::Value, Error> {
//     todo!("APY")
// }

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
    let args: RpcArgs<Option<PaginationParams>> = params.parse()?;
    let (id, pagination) = args.unpack();
    let pagination = pagination.unwrap_or_default();

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::open_orders(&mut conn, id, pagination.limit, pagination.offset) {
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
