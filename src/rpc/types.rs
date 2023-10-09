#![allow(non_camel_case_types)]

// •	Live Price Data
// •	Historical Price Data
// •	Funding Rate
// •	Historical Funding Rate
// •	Open Limit Orders
// •	Recent Trade Orders (24Hours)
// •	24 Hours Pool APY
// •	Market Info
// •	Candle data (Kline data: 1min, 5min, 15min, 30min, 1hr, 4hr, 8hr, 12hr, 24hr)
// •	Position Size (For Long, Short and Total)
// •	Server Time
use crate::auth::AuthInfo;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcArgs<T> {
    pub user: AuthInfo,
    pub params: T,
}

impl<T> RpcArgs<T> {
    pub fn unpack(self) -> (i64, T) {
        let RpcArgs {
            user: AuthInfo { customer_id, .. },
            params,
        } = self;
        (customer_id, params)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderId {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CandlestickResolution {
    Hourly(usize),
    Daily(usize),
    Weekly(usize),
    Monthly(usize),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CandlestickQuery {
    pub from: std::time::SystemTime,
    pub to: Option<std::time::SystemTime>,
    pub resolution: CandlestickResolution,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BTCPrice {
    pub price: f64, //xxxx.xx
    pub effective_at: std::time::SystemTime,
    pub current_time: std::time::SystemTime, //server time
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FundingRate {
    pub rate: f64,
    pub btc_price: f64,
    pub effective_at: std::time::SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Candle {
    pub started_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub resolution: String, //1hour, 4hour, 1 day etc...
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub btc_volume: f64,
    pub trades: i32,
    pub usd_volume: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PositionSize {
    pub total_short_position_size: f64,
    pub total_long_position_size: f64,
    pub total_position_size: f64,
}

// Open Limit Orders
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub bid: Vec<Bid>,
    pub ask: Vec<Ask>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bid {
    pub positionsize: f64,
    pub price: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ask {
    pub positionsize: f64,
    pub price: f64,
}

//******Open Limit Orders end */
//***** Recent Trade Orders (24Hours) */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Side {
    BUY,
    SELL,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CloseTrade {
    pub side: Side,
    pub positionsize: f64,
    pub price: f64,
    pub timestamp: std::time::SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RecentOrders {
    pub orders: Vec<CloseTrade>,
}

//***** Recent Trade Orders (24Hours) end */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ServerTime {
    pub iso: String,
    pub epoch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MarketInfo {
    pub status: String,      //Online/Offline
    pub base_asset: String,  //BTC
    pub quote_asset: String, //USD
    pub step_size: i32,
    pub tick_size: f64,
    pub index_price: f64,
    pub oracle_price: f64,
    pub price_change24h: f64,
    pub next_funding_rate: f64,
    pub next_funding_at: std::time::SystemTime,
    pub min_order_size: f64,
    pub initial_margin_fraction: f64,
    pub maintenance_margin_fraction: f64,
    pub volume24h: f64,
    pub trades24h: i32,
    pub open_interest: i32,
    pub incremental_initial_margin_fraction: f64,
    pub incremental_position_size: f64,
    pub max_position_size: f64,
    pub baseline_position_size: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeVolumeArgs {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderHistoryArgs {
    OrderId(String),
    ClientId { offset: i64, limit: i64 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PnlArgs {
    OrderId(String),
    PublicKey(String),
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Candles {
    pub interval: Interval,
    pub since: DateTime<Utc>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CandleSubscription {
    pub interval: Interval,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Interval {
    ONE_MINUTE,
    FIVE_MINUTE,
    FIFTEEN_MINUTE,
    THIRTY_MINUTE,
    ONE_HOUR,
    FOUR_HOUR,
    EIGHT_HOUR,
    TWELVE_HOUR,
    ONE_DAY,
}

impl Interval {
    pub fn interval_sql(&self) -> String {
        match self {
            Interval::ONE_MINUTE => "'1 minute'",
            Interval::FIVE_MINUTE => "'5 minutes'",
            Interval::FIFTEEN_MINUTE => "'15 minutes'",
            Interval::THIRTY_MINUTE => "'30 minutes'",
            Interval::ONE_HOUR => "'1 hour'",
            Interval::FOUR_HOUR => "'4 hours'",
            Interval::EIGHT_HOUR => "'8 hours'",
            Interval::TWELVE_HOUR => "'12 hours'",
            Interval::ONE_DAY => "'1 day'",
        }
        .into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalPriceArgs {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalFundingArgs {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub limit: i64,
    pub offset: i64,
}
