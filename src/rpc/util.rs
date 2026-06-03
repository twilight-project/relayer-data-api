use crate::database::{Ask, Bid, OrderBook, RecentOrder};
use super::types::{FundingRateResponse, MarketRiskStatsResponse, MarketStatus, RiskParams};
use chrono::{DateTime, TimeDelta, Utc};
use itertools::Itertools;
use relayer_core::relayer::RiskState;

const BOOK_LIMIT: usize = 10;
const RECENT_ORDER_LIMIT: usize = 25;

pub fn order_book(conn: &mut redis::Connection) -> OrderBook {
    let asks: redis::Iter<f64> = redis::cmd("ZSCAN")
        .arg("ask")
        .cursor_arg(0)
        .clone()
        .iter(conn)
        .unwrap();

    let ask: Vec<_> = asks
        .chunks(2)
        .into_iter()
        .take(BOOK_LIMIT)
        .map(|mut chunk| {
            let positionsize = chunk.next().unwrap();
            let price = chunk.next().unwrap() / 100.0;

            Ask {
                id: "".into(),
                positionsize,
                price,
            }
        })
        .collect();

    let bids: redis::Iter<f64> = redis::cmd("ZSCAN")
        .arg("bid")
        .cursor_arg(0)
        .clone()
        .iter(conn)
        .unwrap();

    let bids: Vec<_> = bids.collect();
    let bid: Vec<_> = bids
        .chunks(2)
        .rev()
        .into_iter()
        .take(BOOK_LIMIT)
        .map(|chunk| {
            let positionsize = chunk[0];
            let price = chunk[1] / 100.0;

            Bid {
                id: "".into(),
                positionsize,
                price,
            }
        })
        .collect();

    OrderBook { ask, bid }
}

pub fn recent_orders(conn: &mut redis::Connection) -> Vec<RecentOrder> {
    let max = Utc::now();
    let min = max - TimeDelta::days(1);

    let orders: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg("recent_orders")
        .arg(min.timestamp_millis())
        .arg(max.timestamp_millis())
        .query(conn)
        .unwrap();

    orders
        .into_iter()
        .rev()
        .take(RECENT_ORDER_LIMIT)
        .map(|order| serde_json::from_str(&order).expect("Invalid recent order!"))
        .collect()
}

pub fn compute_market_risk_stats(
    risk_state: &RiskState,
    pool_equity_btc: f64,
    mark_price: f64,
    params: RiskParams,
    funding_rate: f64,
    funding_rate_timestamp: DateTime<Utc>,
    position_long_usd: f64,
    position_short_usd: f64,
) -> MarketRiskStatsResponse {

    // Compute market status
    let (status, status_reason) = if risk_state.manual_halt {
        (MarketStatus::HALT, Some("MANUAL_HALT".to_string()))
    } else if risk_state.manual_close_only {
        (MarketStatus::CLOSE_ONLY, Some("MANUAL_CLOSE_ONLY".to_string()))
    } else if pool_equity_btc <= 0.0 {
        (MarketStatus::HALT, Some("POOL_EQUITY_INVALID".to_string()))
    } else {
        (MarketStatus::HEALTHY, None)
    };

    // Exposure is netted in USD notional (mirrors relayer-core).
    let total_long = risk_state.total_long_usd;
    let total_short = risk_state.total_short_usd;
    let oi_usd = total_long + total_short;
    let net_usd = total_long - total_short;

    // Pool equity expressed in USD at the current mark, so exposure and equity
    // share a unit. A non-positive/invalid mark yields 0 (caps collapse to 0).
    let pool_equity_usd = if mark_price.is_finite() && mark_price > 0.0 {
        pool_equity_btc * mark_price
    } else {
        0.0
    };

    let (long_pct, short_pct) = if oi_usd > 0.0 {
        (total_long / oi_usd, total_short / oi_usd)
    } else {
        (0.0, 0.0)
    };

    let utilization = if pool_equity_usd > 0.0 {
        oi_usd / pool_equity_usd
    } else {
        0.0
    };

    // Compute limits in USD (matching relayer-core compute_limits)
    let oi_max_usd = params.max_oi_mult * pool_equity_usd;
    let net_max_usd = params.max_net_mult * pool_equity_usd;
    let pos_max_usd = params.max_position_pct * pool_equity_usd;

    let x_oi = (oi_max_usd - oi_usd).max(0.0);
    let x_net_long = (net_max_usd - net_usd).max(0.0);
    let x_net_short = (net_max_usd + net_usd).max(0.0);

    let (max_long_usd, max_short_usd) = if status != MarketStatus::HEALTHY {
        (0.0, 0.0)
    } else {
        let max_long = x_oi.min(x_net_long).min(pos_max_usd);
        let max_short = x_oi.min(x_net_short).min(pos_max_usd);
        (max_long, max_short)
    };

    // Estimated next funding rate from the position-size-log USD skew.
    let mut estimated_funding_rate: f64;
    let psi = 1.0;
    if position_long_usd + position_short_usd == 0.0 {
        estimated_funding_rate = 0.0;
    } else {
        estimated_funding_rate = ((position_long_usd - position_short_usd)
            / (position_long_usd + position_short_usd))
            .powi(2)
            / (psi * 8.0);
    }

    //positive funding if totallong > totalshort else negative funding
    if position_long_usd <= position_short_usd {
        estimated_funding_rate = estimated_funding_rate * -1.0;
    }
    estimated_funding_rate = (estimated_funding_rate * 1_000_000.0).round() / 1_000_000.0;

    let estimated_funding_rate_timestamp = funding_rate_timestamp + TimeDelta::hours(1);

    MarketRiskStatsResponse {
        pool_equity_btc,
        pool_equity_usd,
        mark_price,
        total_long_usd: total_long,
        total_short_usd: total_short,
        total_pending_long_usd: risk_state.total_pending_long_usd,
        total_pending_short_usd: risk_state.total_pending_short_usd,
        open_interest_usd: oi_usd,
        net_exposure_usd: net_usd,
        long_pct,
        short_pct,
        utilization,
        max_long_usd,
        max_short_usd,
        status,
        status_reason,
        params,
        funding_rate: FundingRateResponse {
            funding_rate,
            estimated_funding_rate,
            funding_rate_timestamp,
            estimated_funding_rate_timestamp,
        },
    }
}
