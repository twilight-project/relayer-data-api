use crate::database::{Ask, Bid, OrderBook, RecentOrder};
use super::types::{MarketRiskStatsResponse, MarketStatus, RiskParams};
use chrono::{TimeDelta, Utc};
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
) -> MarketRiskStatsResponse {
    let params = RiskParams::from_env();

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

    let total_long = risk_state.total_long_btc;
    let total_short = risk_state.total_short_btc;
    let oi_btc = total_long + total_short;
    let net_btc = total_long - total_short;

    let (long_pct, short_pct) = if oi_btc > 0.0 {
        (total_long / oi_btc, total_short / oi_btc)
    } else {
        (0.0, 0.0)
    };

    let utilization = if pool_equity_btc > 0.0 {
        oi_btc / pool_equity_btc
    } else {
        0.0
    };

    // Compute limits (matching relayer-core compute_limits)
    let oi_max_btc = params.max_oi_mult * pool_equity_btc;
    let net_max_btc = params.max_net_mult * pool_equity_btc;
    let pos_max_btc = params.max_position_pct * pool_equity_btc;

    let x_oi = (oi_max_btc - oi_btc).max(0.0);
    let x_net_long = (net_max_btc - net_btc).max(0.0);
    let x_net_short = (net_max_btc + net_btc).max(0.0);

    let (max_long_btc, max_short_btc) = if status != MarketStatus::HEALTHY {
        (0.0, 0.0)
    } else {
        let max_long = x_oi.min(x_net_long).min(pos_max_btc);
        let max_short = x_oi.min(x_net_short).min(pos_max_btc);
        (max_long, max_short)
    };

    MarketRiskStatsResponse {
        pool_equity_btc,
        total_long_btc: total_long,
        total_short_btc: total_short,
        open_interest_btc: oi_btc,
        net_exposure_btc: net_btc,
        long_pct,
        short_pct,
        utilization,
        max_long_btc,
        max_short_btc,
        status,
        status_reason,
        params,
    }
}
