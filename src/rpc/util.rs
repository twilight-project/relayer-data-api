use crate::database::{Ask, Bid, OrderBook, RecentOrder};
use chrono::{TimeDelta, Utc};
use itertools::Itertools;

const BOOK_LIMIT: usize = 10;

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
        .map(|order| serde_json::from_str(&order).expect("Invalid recent order!"))
        .collect()
}
