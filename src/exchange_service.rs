use crate::orderbook::orderbook_aggregator_server::*;
use crate::orderbook::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::watch;
// use tokio::sync::watch::*;
use tokio::sync::mpsc;
// use tokio::sync::mpsc::Receiver;
use crate::exchange::OrderBookUpdate;
use tokio_stream::wrappers::ReceiverStream;
// use crate::exchange::OrderBookData;
use crate::orderbook::Summary;

// todo make configurable
const TOP_N: usize = 10;

// maintains the last order book Summary for any number of exchanges.
// This allows generation of a summary w/ top 10 bids/asks across exchanges
struct OrderBookData {
    // stores the last update from each exchange.
    exchange_data: HashMap<String, OrderBookUpdate>,
}

impl OrderBookData {
    /// replaces the data from a specific exchange.
    pub fn update_exchange_data(&mut self, update: OrderBookUpdate) {
        // TODO keep only top n on insert to make summary faster to build.
        self.exchange_data.insert(update.exchange.clone(), update);
    }

    /// summary returns a Summary containing top 10 bids/asks across all exchanges.
    /// TODO can be made more efficient via k-way merge eg using vec like a minheap.
    /// (For the current requirement, this will be sufficient.)
    pub fn summary(&self) -> Summary {
        let mut bids = vec![];
        let mut asks = vec![];

        for (_ex, ex_summary) in self.exchange_data.borrow().into_iter() {
            // For the sake of simplicity, we clone the bids and asks. Only TOP_N are kept per exchange.
            bids.append(&mut ex_summary.bids.clone());
            asks.append(&mut ex_summary.asks.clone());
        }

        bids.sort_by(|a, b| {
            if a.price < b.price {
                Ordering::Less
            } else if a.price == b.price {
                Ordering::Equal
            } else {
                Ordering::Greater
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

impl Summary {
    // merges new order book data, returning a new Summary with _ALL ORDERBOOK_DATA.
    // Use truncate to get the actual summary.
    // // pub fn merge(&self, mut order_book_data: OrderBookUpdate) -> Summary {
    // //     let mut exchange = "".to_string();
    // //
    // //     match (order_book_data.bids.first(), order_book_data.asks.first()) {
    // //         (Some(bid), _) => {
    // //             exchange = bid.exchange.clone();
    // //         }
    // //         (_, Some(ask)) => {
    // //             exchange = ask.exchange.clone();
    // //         }
    // //         _ => {
    // //             println!("no data!");
    // //             return self.clone()
    // //         }, // no changes so return early
    // //     }
    // //
    // //     println!("exchange: {}", exchange.clone());
    // //
    // //     let mut new_summary = Summary{
    // //         spread: 0.0,
    // //         bids: self.bids.clone()//TODO remove clone...
    // //         .into_iter()
    // //         .filter(|o| {
    // //             println!("equals? {} {} {}", o.exchange.eq(&exchange), o.exchange, &exchange);
    // //             o.exchange.eq(&exchange)
    // //         })
    // //         .collect(),
    // //         asks: self.bids.clone() //TODO remove clone...
    // //         .into_iter()
    // //         .filter(|o| o.exchange.eq(&exchange))
    // //         .collect()
    // //     };
    // //
    // //     // todo abstract/deduplicate this logic
    // //
    // //     // sorts in order and takes first TOP_N. Nothing fancy.
    // //
    // //     // Commentary on perf:
    // //     // - would be possible to sort the new data and merge with existing sorted summary but the benefit is small.
    // //     // - Should be roughly O(2n*logn) for append + sort as append moves the elements efficiently.
    // //     // - Biggest cost is probably comparing the exchange to filter old values
    // //     new_summary.bids.append(
    // //         &mut order_book_data.bids
    // //     );
    // //     new_summary.bids.sort_by(|a, b| {
    // //         if a.price < b.price {
    // //             Ordering::Less
    // //         } else if a.price == b.price {
    // //             Ordering::Equal
    // //         } else {
    // //             Ordering::Greater
    // //         }
    // //     });
    // //
    // //     new_summary.asks.append(&mut order_book_data.asks);
    // //     new_summary.asks.sort_by(|a, b| {
    // //         if a.price < b.price {
    // //             Ordering::Less
    // //         } else if a.price == b.price {
    // //             Ordering::Equal
    // //         } else {
    // //             Ordering::Greater
    // //         }
    // //     });
    // //
    // //     // calculate the new spread based on the first bid and ask price (they're the best) o(n)
    // //     // we check there are bids and asks or else we leave the spread
    // //     if !new_summary.asks.is_empty() && !new_summary.bids.is_empty() {
    // //         new_summary.spread =
    // //             new_summary.asks.first().unwrap().price - new_summary.bids.first().unwrap().price;
    // //     }
    // //
    // //     if new_summary.spread < 0.0 {
    // //         // TODO delete.
    // //         info!(
    // //             "A negative spread was observed while merging new order book data from {}",
    // //             exchange
    // //         );
    // //     }
    // //
    // //     new_summary
    // }

    // returns only TOP_N
    // pub fn truncate_top_n(&self) -> Summary {
    //     let mut new_summary = self.clone();
    //     new_summary.bids.truncate(TOP_N);
    //     new_summary.bids.truncate(TOP_N);
    //     new_summary
    // }
}

pub struct AggregatorProcess {
    // receives new order book data from exchanges
    pub(crate) exchange_rx: mpsc::Receiver<OrderBookUpdate>,
    // sends the updated summary to a watch for clients
    pub(crate) watch_tx: watch::Sender<Summary>,
    last_summary: Summary,
}

impl AggregatorProcess {
    pub async fn start(
        exchange_rx: mpsc::Receiver<OrderBookUpdate>,
        watch_tx: watch::Sender<Summary>,
    ) {
        tokio::spawn(async move {
            let mut agg = AggregatorProcess {
                exchange_rx,
                watch_tx,
                last_summary: Summary {
                    spread: 0.0,
                    bids: vec![],
                    asks: vec![],
                },
            };

            loop {
                match agg.exchange_rx.recv().await {
                    None => debug!("empty exchange update received on exchange channel"),
                    Some(order_book_data) => {}
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct OrderbookAggregatorServer {
    pub(crate) watch_rx: watch::Receiver<Summary>,
    // last_summary: Summary,
    // subscribers: Vec<Receiver<Result<Summary, tonic::Status>>>
}

impl OrderbookAggregatorServer {
    pub fn new(watch_rx: watch::Receiver<Summary>) -> OrderbookAggregatorServer {
        OrderbookAggregatorServer {
            watch_rx,
            // last_summary: Summary{
            //     spread: 0.0,
            //     bids: vec![],
            //     asks: vec![]
            // },
            // subscribers: vec![]
        }
    }
}

type SummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, tonic::Status>> + Send + 'static>>;

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorServer {
    type BookSummaryStream = ReceiverStream<Result<Summary, tonic::Status>>;

    async fn book_summary(
        &self,
        request: tonic::Request<crate::orderbook::Empty>,
    ) -> Result<tonic::Response<Self::BookSummaryStream>, tonic::Status> {
        println!("book summary");
        let (mut tx, rx) = mpsc::channel(4);

        let mut wrx: tokio::sync::watch::Receiver<Summary> = self.watch_rx.clone();

        tokio::spawn(async move {
            loop {
                // println!("looping");
                // println!("looping {:?}", wrx.borrow().spread);
                wrx.changed().await.unwrap(); // FIXME unsafe result.

                let val = wrx.borrow().clone();
                tx.send(Ok(val)).await; // FIXME move await. also unused result.
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    // use crate::app_config::AppConfig;
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
                price: 4.4,
                amount: 11.0,
            }
        );

        assert_eq!(
            summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 7.7,
                amount: 9.0,
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
        assert_eq!(summary.spread, 40.4 - 4.4)
    }

    #[test]
    fn should_replace_new_order_book_data_for_previously_seen_exchange() {
        let mut order_book_data = OrderBookData {
            exchange_data: Default::default(),
        };

        let order_book_update = OrderBookUpdate {
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
                price: 4.5,
                amount: 11.0,
            }
        );

        assert_eq!(
            summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 7.8,
                amount: 9.0,
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
        assert_eq!(summary.spread, 41.4 - 4.5)
    }

    #[test]
    fn should_handle_order_book_data_for_multiple_exchanges() {
        let mut order_book_data = OrderBookData {
            exchange_data: Default::default(),
        };

        let order_book_update = OrderBookUpdate {
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
                exchange: "binance".to_string(),
                price: 4.4,
                amount: 11.0,
            }
        );

        assert_eq!(
            summary.bids.last().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 7.8,
                amount: 9.0,
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
        assert_eq!(summary.spread, 40.4 - 4.4)
    }
}
