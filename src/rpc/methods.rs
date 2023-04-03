use super::RelayerContext;
use chrono::prelude::*;
use crate::database::*;
use jsonrpsee::{core::error::Error, server::logger::Params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderId {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalPriceArgs {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    //TODO: paginate?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalFundingArgs {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    //TODO: paginate?
}

pub(super) fn trader_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let OrderId { id } = params.parse()?;

    match ctx.pool.get() {
        Ok(mut conn) => match TraderOrder::get(&mut conn, id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

pub(super) fn lend_order_info(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let OrderId { id } = params.parse()?;

    match ctx.pool.get() {
        Ok(mut conn) => match LendOrder::get(&mut conn, id) {
            Ok(o) => Ok(serde_json::to_value(o).expect("Error converting response")),
            Err(e) => Err(Error::Custom(format!("Error fetching order info: {:?}", e))),
        },
        Err(e) => Err(Error::Custom(format!("Database error: {:?}", e))),
    }
}

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
    let args = match params.parse::<Option<HistoricalPriceArgs>>() {
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

pub(super) fn historical_funding_rate(
    params: Params<'_>,
    ctx: &RelayerContext,
) -> Result<serde_json::Value, Error> {
    let args = match params.parse::<Option<HistoricalFundingArgs>>() {
        Ok(args) => args,
        Err(e) => return Err(Error::Custom(format!("Invalid argument: {:?}", e))),
    };

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
        Ok(mut conn) => match TraderOrder::list_open_limit_orders(&mut conn) {
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
