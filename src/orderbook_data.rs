//! contains the logic in the application that merges OrderBookUpdates for different exchanges
//! and will provide a Summary with TOP_N bids/asks and the spread across all exchange data.
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::exchange::OrderBookUpdate;
use crate::orderbook::Summary;

const TOP_N: usize = 10; // Determines how many bids/asks are kept.

// maintains the last order book Summary for any number of exchanges.
// This allows generation of a summary w/ top 10 bids/asks across exchanges
#[derive(Default)]
pub struct OrderBookData {
    exchange_data: HashMap<String, OrderBookUpdate>,
}

impl OrderBookData {
    /// replaces the data from a specific exchange.
    pub fn update_exchange_data(&mut self, update: OrderBookUpdate) {
        self.exchange_data.insert(update.exchange.clone(), update);
    }

    /// summary returns a Summary containing top 10 bids/asks across all exchanges.
    /// can be made more efficient via k-way merge eg using vec like a minheap.
    /// (For the current requirement, this will be sufficient.)
    pub fn summary(&self) -> Summary {
        let mut bids = vec![];
        let mut asks = vec![];

        for (_ex, ex_summary) in self.exchange_data.borrow().into_iter() {
            // For the sake of simplicity, we clone the bids and asks. Only TOP_N are kept per exchange.
            bids.append(&mut ex_summary.bids.clone());
            asks.append(&mut ex_summary.asks.clone());
        }

        // Note: bids and asks are sorted inversely from each other.
        bids.sort_by(|a, b| {
            if a.price < b.price {
                Ordering::Greater
            } else if a.price == b.price {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        });
        bids.truncate(TOP_N);

        asks.sort_by(|a, b| {
            if a.price < b.price {
                Ordering::Less
            } else if a.price == b.price {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });
        asks.truncate(TOP_N);

        let mut spread = 0.0;
        // calculate the new spread based on the first bid and ask price (they're the best) o(n)
        // we check there are bids and asks or else we leave the spread
        if !asks.is_empty() && !bids.is_empty() {
            spread = asks.first().unwrap().price - bids.first().unwrap().price;
        }

        Summary { spread, bids, asks }
    }
}

#[cfg(test)]
mod tests {
    use crate::orderbook::Level;
    use tokio::time::Instant;

    use super::*;

    // test helper to take pairs of (price, amount) and return vector of levels.
    // Duplication is okay in tests, but this reduces noise a bit.
    fn sample_levels(exchange: String, price_and_amount: Vec<(f64, f64)>) -> Vec<Level> {
        let mut levels = vec![];

        for (price, amount) in price_and_amount {
            let level = Level {
                exchange: exchange.clone(),
                price,
                amount,
            };
            levels.push(level);
        }

        levels
    }

    #[test]
    fn should_add_new_order_book_data_for_unseen_exchange() {
        let mut order_book_data = OrderBookData {
            exchange_data: Default::default(),
        };

        let order_book_update = OrderBookUpdate {
            ts: Instant::now(),
            exchange: "binance".to_string(),
            bids: sample_levels(
                "binance".to_string(),
                vec![(5.5, 10.0), (4.4, 11.0), (7.7, 9.0), (6.6, 8.0)],
            ),
            asks: sample_levels(
                "binance".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                ],
            ),
        };

        order_book_data.update_exchange_data(order_book_update);
        let summary = order_book_data.summary();

        assert_eq!(summary.bids.len(), 4);
        // top bid
        assert_eq!(
            summary.bids.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 7.7,
                amount: 9.0,
            }
        );

        assert_eq!(
            summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 4.4,
                amount: 11.0,
            }
        );

        assert_eq!(summary.asks.len(), 5);
        // top ask
        assert_eq!(
            summary.asks.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 40.4,
                amount: 110.0,
            }
        );

        assert_eq!(
            summary.asks.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 70.7,
                amount: 90.0,
            }
        );

        // top ask - top bid.
        assert_eq!(summary.spread, 40.4 - 7.7)
    }

    #[test]
    fn should_replace_new_order_book_data_for_previously_seen_exchange() {
        let mut order_book_data = OrderBookData {
            exchange_data: Default::default(),
        };

        let order_book_update = OrderBookUpdate {
            ts: Instant::now(),
            exchange: "binance".to_string(),
            bids: sample_levels(
                "binance".to_string(),
                vec![(5.5, 10.0), (4.4, 11.0), (7.7, 9.0), (6.6, 8.0)],
            ),
            asks: sample_levels(
                "binance".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                ],
            ),
        };

        order_book_data.update_exchange_data(order_book_update);

        let order_book_update = OrderBookUpdate {
            ts: Instant::now(),
            exchange: "binance".to_string(),
            bids: sample_levels(
                "binance".to_string(),
                vec![(5.6, 10.0), (4.5, 11.0), (7.8, 9.0), (6.7, 8.0)],
            ),
            asks: sample_levels(
                "binance".to_string(),
                vec![
                    (51.5, 100.0),
                    (41.4, 110.0),
                    (71.7, 90.0),
                    (61.6, 80.0),
                    (56.6, 80.0),
                ],
            ),
        };

        order_book_data.update_exchange_data(order_book_update);

        let summary = order_book_data.summary();

        assert_eq!(summary.bids.len(), 4);
        // top bid
        assert_eq!(
            summary.bids.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 7.8,
                amount: 9.0,
            }
        );
        assert_eq!(
            summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 4.5,
                amount: 11.0,
            }
        );
        assert_eq!(summary.asks.len(), 5);

        // top ask
        assert_eq!(
            summary.asks.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 41.4,
                amount: 110.0,
            }
        );

        assert_eq!(
            summary.asks.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 71.7,
                amount: 90.0,
            }
        );

        // top ask - top bid.
        assert_eq!(summary.spread, 41.4 - 7.8)
    }

    #[test]
    fn should_handle_order_book_data_for_multiple_exchanges() {
        let mut order_book_data = OrderBookData {
            exchange_data: Default::default(),
        };

        let order_book_update = OrderBookUpdate {
            ts: Instant::now(),
            exchange: "binance".to_string(),
            bids: sample_levels(
                "binance".to_string(),
                vec![(5.5, 10.0), (4.4, 11.0), (7.7, 9.0), (6.6, 8.0)],
            ),
            asks: sample_levels(
                "binance".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                ],
            ),
        };

        order_book_data.update_exchange_data(order_book_update);

        let order_book_update = OrderBookUpdate {
            ts: Instant::now(),
            exchange: "bitstamp".to_string(),
            bids: sample_levels(
                "bitstamp".to_string(),
                vec![(5.6, 10.0), (4.5, 11.0), (7.8, 9.0), (6.7, 8.0)],
            ),
            asks: sample_levels(
                "bitstamp".to_string(),
                vec![
                    (51.5, 100.0),
                    (41.4, 110.0),
                    (71.7, 90.0),
                    (61.6, 80.0),
                    (56.6, 80.0),
                ],
            ),
        };

        order_book_data.update_exchange_data(order_book_update);

        let summary = order_book_data.summary();

        assert_eq!(summary.bids.len(), 8);
        // top bid
        assert_eq!(
            summary.bids.first().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 7.8,
                amount: 9.0,
            }
        );
        assert_eq!(
            summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 4.4,
                amount: 11.0,
            }
        );

        assert_eq!(summary.asks.len(), 10);
        // top ask
        assert_eq!(
            summary.asks.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 40.4,
                amount: 110.0,
            }
        );

        assert_eq!(
            summary.asks.last().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 71.7,
                amount: 90.0,
            }
        );

        // top ask - top bid.
        assert_eq!(summary.spread, 40.4 - 7.8)
    }
}
