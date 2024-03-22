use super::*;
use crate::database::*;
use chrono::prelude::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use relayerwalletlib::verify_client_message::verify_query_order;
use twilight_relayer_rust::relayer;
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
    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::order_book(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn recent_trade_orders(
    _: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::list_past_24hrs(&mut conn) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
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
