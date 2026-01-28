#![allow(non_camel_case_types)]
#![allow(warnings)]
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
// •	Fee History
use crate::auth::UserInfo;
use crate::database::OrderStatus;
use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcArgs<T> {
    pub user: UserInfo,
    pub params: T,
}

impl<T> RpcArgs<T> {
    pub fn unpack(self) -> (i64, T) {
        let RpcArgs {
            user: UserInfo { customer_id },
            params,
        } = self;
        (customer_id, params)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub data: String,
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
pub enum TransactionHashArgs {
    TxId {
        id: String,
        status: Option<OrderStatus>,
    },
    AccountId {
        id: String,
        status: Option<OrderStatus>,
    },
    RequestId {
        id: String,
        status: Option<OrderStatus>,
    },
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
pub enum LendOrderArgs {
    OrderId(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderHistoryArgs {
    OrderId(String),
    ClientId {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        offset: i64,
        limit: i64,
    },
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

#[derive(Copy, Eq, Hash, PartialEq, Clone, Debug, Serialize, Deserialize)]
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
    ONE_DAY_CHANGE,
}

impl Interval {
    pub fn duration(&self) -> Duration {
        match self {
            Interval::ONE_MINUTE => Duration::minutes(1),
            Interval::FIVE_MINUTE => Duration::minutes(5),
            Interval::FIFTEEN_MINUTE => Duration::minutes(15),
            Interval::THIRTY_MINUTE => Duration::minutes(30),
            Interval::ONE_HOUR => Duration::hours(1),
            Interval::FOUR_HOUR => Duration::hours(4),
            Interval::EIGHT_HOUR => Duration::hours(8),
            Interval::TWELVE_HOUR => Duration::hours(12),
            Interval::ONE_DAY => Duration::days(1),
            Interval::ONE_DAY_CHANGE => Duration::days(1),
        }
    }

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
            Interval::ONE_DAY_CHANGE => "'1 day'",
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FeeHistory {
    pub order_filled_on_market: f64,
    pub order_filled_on_limit: f64,
    pub order_settled_on_market: f64,
    pub order_settled_on_limit: f64,
    pub updated_at: DateTime<Utc>,
}
// impl FeeHistory {
//     pub fn new(
//         order_filled_on_market: f64,
//         order_filled_on_limit: f64,
//         order_settled_on_market: f64,
//         order_settled_on_limit: f64,
//         updated_at: DateTime<Utc>,
//     ) -> Self {
//         Self {
//             order_filled_on_market,
//             order_filled_on_limit,
//             order_settled_on_market,
//             order_settled_on_limit,
//             updated_at,
//         }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalFeeArgs {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApySeriesArgs {
    // Range of the chart. Supported: "1d" | "7d" | "30d" | explicit like "24 hours", "7 days", "30 days"
    pub range: String,
    // Optional step override. Examples: "1m","5m","15m","30m","1h","2h","4h","12h"
    #[serde(default)]
    pub step: Option<String>,
    // Optional lookback window for trailing APY. Default "24 hours".
    #[serde(default)]
    pub lookback: Option<String>,
}

impl ApySeriesArgs {
    /// Map friendly tokens to Postgres interval strings.
    fn normalize_interval(s: &str) -> Option<&'static str> {
        match s.trim().to_lowercase().as_str() {
            // ranges
            "1d" | "1day" | "24h" => Some("24 hours"),
            "7d" | "1w" | "7days" => Some("7 days"),
            "30d" | "1m" | "30days" => Some("30 days"),
            // steps (minutes)
            "1m" => Some("1 minute"),
            "5m" => Some("5 minutes"),
            "15m" => Some("15 minutes"),
            "30m" => Some("30 minutes"),
            // steps (hours)
            "1h" => Some("1 hour"),
            "2h" => Some("2 hours"),
            "4h" => Some("4 hours"),
            "12h" => Some("12 hours"),
            // lookbacks
            "24hours" => Some("24 hours"),
            "7days" => Some("7 days"),
            "30days" => Some("30 days"),
            _ => None,
        }
    }

    /// Resolve (window, step, lookback) to Postgres interval strings.
    pub fn resolve(&self) -> Result<(&'static str, &'static str, &'static str), String> {
        // window
        let window = Self::normalize_interval(&self.range)
            .or_else(|| {
                // allow explicit "24 hours" style input
                let s = self.range.trim().to_lowercase();
                match s.as_str() {
                    "24 hours" | "7 days" | "30 days" => Some(Box::leak(s.into_boxed_str())),
                    _ => None,
                }
            })
            .ok_or_else(|| format!("Unsupported range: {}", self.range))?;

        // default step per window if none provided
        let default_step = match window {
            "24 hours" => "1 minute",
            "7 days" => "5 minutes",
            "30 days" => "1 hour",
            _ => "1 minute",
        };

        // step
        let step = if let Some(ref s) = self.step {
            Self::normalize_interval(s)
                .or_else(|| {
                    let s = s.trim().to_lowercase();
                    match s.as_str() {
                        "1 minute" | "5 minutes" | "15 minutes" | "30 minutes" | "1 hour"
                        | "2 hours" | "4 hours" | "12 hours" => Some(Box::leak(s.into_boxed_str())),
                        _ => None,
                    }
                })
                .ok_or_else(|| format!("Unsupported step: {}", s))?
        } else {
            default_step
        };

        // lookback
        let lookback = if let Some(ref lb) = self.lookback {
            Self::normalize_interval(lb)
                .or_else(|| {
                    let s = lb.trim().to_lowercase();
                    match s.as_str() {
                        "24 hours" | "7 days" | "30 days" => Some(Box::leak(s.into_boxed_str())),
                        _ => None,
                    }
                })
                .ok_or_else(|| format!("Unsupported lookback: {}", lb))?
        } else {
            "24 hours"
        };

        Ok((window, step, lookback))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSummaryByTAddressArgs {
    pub t_address: String,

    #[serde(default)]
    pub from: Option<DateTime<Utc>>,

    #[serde(default)]
    pub to: Option<DateTime<Utc>>,

    #[serde(default)]
    pub since: Option<DateTime<Utc>>,
}
impl AccountSummaryByTAddressArgs {
    pub fn unpack(
        self,
    ) -> (
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
    ) {
        let AccountSummaryByTAddressArgs {
            t_address,
            from,
            to,
            since,
        } = self;
        (t_address, from, to, since)
    }

    pub fn normalize(self) -> Result<(String, DateTime<Utc>, DateTime<Utc>), String> {
        let now = Utc::now();
        let min_allowed = now - Duration::days(7);

        let AccountSummaryByTAddressArgs {
            t_address,
            from,
            to,
            since,
        } = self;

        // -------------------------------
        // Case 1: since is provided
        // -------------------------------
        if let Some(since_ts) = since {
            if since_ts > min_allowed {
                return Err(
                    "`since` must be at least 7 days older than the current time.".to_string(),
                );
            }

            return Ok((t_address, since_ts, min_allowed));
        }

        // -------------------------------
        // Case 2: from / to logic
        // -------------------------------
        match (from, to) {
            (Some(from_ts), Some(to_ts)) => {
                if from_ts > min_allowed {
                    return Err(
                        "`from` must be at least 7 days older than the current time.".to_string(),
                    );
                }

                let normalized_to = if to_ts > now { now } else { to_ts };

                Ok((t_address, from_ts, normalized_to))
            }

            (Some(from_ts), None) => {
                if from_ts > min_allowed {
                    return Err(
                        "`from` must be at least 7 days older than the current time.".to_string(),
                    );
                }

                Ok((t_address, from_ts, min_allowed))
            }

            (None, Some(_)) => Err("`to` cannot be provided without `from`".to_string()),

            (None, None) => Err("Either `since` or `from` must be provided".to_string()),
        }
    }
}
