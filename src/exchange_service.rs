use crate::orderbook::orderbook_aggregator_server::*;
use crate::orderbook::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::cmp::Ordering;
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
const TOP_N: u32 = 10;

impl Summary {
    // merges new order book data, returning a new Summary w/ top 10.
    pub fn merge(&self, mut order_book_data: OrderBookUpdate) -> Summary {
        let mut exchange = "".to_string();

        match (order_book_data.bids.first(), order_book_data.asks.first()) {
            (Some(bid), _) => {
                exchange = bid.exchange.clone();
            }
            (_, Some(ask)) => {
                exchange = ask.exchange.clone();
            }
            _ => return self.clone(), // no changes so return early
        }

        let mut new_summary = self.clone();

        // todo abstract/deduplicate this logic

        // sorts in order and takes first TOP_N. Nothing fancy.

        // Commentary on perf:
        // - would be possible to sort the new data and merge with existing sorted summary but the benefit is small.
        // - Should be roughly O(2n*logn) for append + sort as append moves the elements efficiently.
        // - Biggest cost is probably comparing the exchange to filter old values
        new_summary.bids.append(
            &mut order_book_data
                .bids
                .into_iter()
                .filter(|o| o.exchange.eq(&exchange))
                .collect(),
        );
        new_summary.bids.sort_by(|a, b| {
            if a.price < b.price {
                Ordering::Less
            } else if a.price == b.price {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });
        new_summary.bids.truncate(10);

        new_summary.asks.append(&mut order_book_data.asks);
        new_summary.asks.sort_by(|a, b| {
            if a.price < b.price {
                Ordering::Less
            } else if a.price == b.price {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });
        new_summary.asks.truncate(10);

        // calculate the new spread based on the first bid and ask price (they're the best) o(n)
        // we check there are bids and asks or else we leave the spread
        if !new_summary.asks.is_empty() && !new_summary.bids.is_empty() {
            new_summary.spread =
                new_summary.asks.first().unwrap().price - new_summary.bids.first().unwrap().price;
        }

        if new_summary.spread < 0.0 {
            // TODO delete.
            info!(
                "A negative spread was observed while merging new order book data from {}",
                exchange
            );
        }

        new_summary
    }
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
    fn summary_should_merge_new_order_book_data_when_empty() {
        let summary = Summary {
            spread: 0.0,
            bids: vec![],
            asks: vec![],
        };

        let order_book_data = OrderBookUpdate {
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

        let updated_summary = summary.merge(order_book_data);

        assert_eq!(updated_summary.bids.len(), 4);
        // top bid
        assert_eq!(
            updated_summary.bids.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 4.4,
                amount: 11.0
            }
        );

        assert_eq!(
            updated_summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 7.7,
                amount: 9.0
            }
        );

        assert_eq!(updated_summary.asks.len(), 5);
        // top ask
        assert_eq!(
            updated_summary.asks.first().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 40.4,
                amount: 110.0
            }
        );

        assert_eq!(
            updated_summary.asks.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 70.7,
                amount: 90.0
            }
        );

        // top ask - top bid.
        assert_eq!(updated_summary.spread, 40.4 - 4.4)
    }

    #[test]
    fn summary_should_merge_new_order_book_data_when_non_empty() {
        let summary = Summary {
            spread: 0.0,
            bids: vec![],
            asks: vec![],
        };

        let order_book_data = OrderBookUpdate {
            bids: sample_levels(
                "binance".to_string(),
                vec![
                    (5.5, 10.0),
                    (4.4, 11.0),
                    (4.5, 11.0),
                    (4.6, 11.0),
                    (4.7, 11.0),
                    (5.7, 11.0),
                    (6.1, 11.0),
                    (6.2, 11.0),
                    (5.9, 11.0),
                    (7.8, 9.0),
                    (6.6, 8.0),
                ],
            ),
            asks: sample_levels(
                "binance".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                    (51.5, 100.0),
                    (41.4, 110.0),
                    (71.7, 90.0),
                    (61.6, 80.0),
                    (56.6, 80.0),
                ],
            ),
        };

        let updated_summary = summary.merge(order_book_data);

        let order_book_data = OrderBookUpdate {
            bids: sample_levels(
                "bitstamp".to_string(),
                vec![
                    (5.5, 10.0),
                    (4.3, 11.1),
                    (4.4, 11.0),
                    (4.5, 11.0),
                    (4.6, 11.0),
                    (4.7, 11.0),
                    (5.7, 11.0),
                    (6.1, 11.0),
                    (6.2, 11.0),
                    (5.9, 11.0),
                    (7.7, 9.0),
                    (6.6, 8.0),
                ],
            ),
            asks: sample_levels(
                "bitstamp".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.1, 110.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                    (51.5, 100.0),
                    (41.4, 110.0),
                    (71.7, 90.0),
                    (61.6, 80.0),
                    (56.6, 80.0),
                ],
            ),
        };

        let updated_summary = updated_summary.merge(order_book_data);

        assert_eq!(updated_summary.bids.len(), 10);
        // top bid
        assert_eq!(
            updated_summary.bids.first().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 4.3,
                amount: 11.1
            }
        );

        assert_eq!(
            updated_summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 5.5,
                amount: 10.0
            }
        );

        assert_eq!(updated_summary.asks.len(), 10);
        // top ask
        assert_eq!(
            updated_summary.asks.first().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 40.1,
                amount: 110.0
            }
        );

        assert_eq!(
            updated_summary.asks.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 55.6,
                amount: 80.0
            }
        );

        // top ask - top bid.
        assert_eq!(updated_summary.spread, 40.1 - 4.3)
    }

    #[test]
    fn summary_should_filter_exchange_data_when_re_adding_data() {
        let summary = Summary {
            spread: 0.0,
            bids: vec![],
            asks: vec![],
        };

        let order_book_data = OrderBookUpdate {
            bids: sample_levels(
                "binance".to_string(),
                vec![
                    (5.5, 10.0),
                    (4.4, 11.0),
                    (4.5, 11.0),
                    (4.6, 11.0),
                    (4.7, 11.0),
                    (5.7, 11.0),
                    (6.1, 11.0),
                    (6.2, 11.0),
                    (5.9, 11.0),
                    (7.8, 9.0),
                    (6.6, 8.0),
                ],
            ),
            asks: sample_levels(
                "binance".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                    (51.5, 100.0),
                    (41.4, 110.0),
                    (71.7, 90.0),
                    (61.6, 80.0),
                    (56.6, 80.0),
                ],
            ),
        };

        let updated_summary = summary.merge(order_book_data);

        let order_book_data = OrderBookUpdate {
            bids: sample_levels(
                "bitstamp".to_string(),
                vec![
                    (5.5, 10.0),
                    (4.3, 11.1),
                    (4.4, 11.0),
                    (4.5, 11.0),
                    (4.6, 11.0),
                    (4.7, 11.0),
                    (5.7, 11.0),
                    (6.1, 11.0),
                    (6.2, 11.0),
                    (5.9, 11.0),
                    (7.7, 9.0),
                    (6.6, 8.0),
                ],
            ),
            asks: sample_levels(
                "bitstamp".to_string(),
                vec![
                    (50.5, 100.0),
                    (40.1, 110.0),
                    (40.4, 110.0),
                    (70.7, 90.0),
                    (60.6, 80.0),
                    (55.6, 80.0),
                    (51.5, 100.0),
                    (41.4, 110.0),
                    (71.7, 90.0),
                    (61.6, 80.0),
                    (56.6, 80.0),
                ],
            ),
        };

        let updated_summary = updated_summary.merge(order_book_data);

        assert_eq!(updated_summary.bids.len(), 10);
        // top bid
        assert_eq!(
            updated_summary.bids.first().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 4.3,
                amount: 11.1
            }
        );

        assert_eq!(
            updated_summary.bids.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 5.5,
                amount: 10.0
            }
        );

        assert_eq!(updated_summary.asks.len(), 10);
        // top ask
        assert_eq!(
            updated_summary.asks.first().unwrap(),
            &Level {
                exchange: "bitstamp".to_string(),
                price: 40.1,
                amount: 110.0
            }
        );

        assert_eq!(
            updated_summary.asks.last().unwrap(),
            &Level {
                exchange: "binance".to_string(),
                price: 55.6,
                amount: 80.0
            }
        );

        // top ask - top bid.
        assert_eq!(updated_summary.spread, 40.1 - 4.3)
    }
}
