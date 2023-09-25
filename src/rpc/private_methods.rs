use super::*;
use crate::database::*;
use chrono::prelude::*;
use jsonrpsee::{core::error::Error, server::logger::Params};

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

    match ctx.pool.get() {
        Ok(mut conn) => match LendOrder::get(&mut conn, args.params.id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn historical_funding_rate(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args = match params.parse::<RpcArgs<HistoricalFundingArgs>>() {
        Ok(args) => args,
        Err(e) => return Err(Error::Custom(format!("Invalid argument: {:?}", e))),
    };

    match ctx.pool.get() {
        Ok(mut conn) => match FundingRate::get_historical(&mut conn, args.params) {
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
    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::unrealized_pnl(&mut conn, args.params) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching pnl: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn open_orders(_: Params<'_>, ctx: &RelayerContext) -> Result<serde_json::Value, Error> {
    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::open_orders(&mut conn) {
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
    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::order_history(&mut conn, args.params) {
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
    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::order_volume(&mut conn, args.params) {
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
