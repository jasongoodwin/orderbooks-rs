//! Contains the details for the async processes that receive and publish updates.
//! See OrderBookData for merging updates and producing summary.
use std::time::Instant;

use metrics::histogram;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio_stream::wrappers::ReceiverStream;

use crate::exchange::OrderBookUpdate;
use crate::orderbook::orderbook_aggregator_server::OrderbookAggregator;
use crate::orderbook::Summary;
use crate::orderbook_data::OrderBookData;

pub struct OrderbookSummaryPublisher {
    // receives new order book data from exchanges
    // sends the updated summary to a watch for clients
    watch_rx: watch::Receiver<Summary>,
}

impl OrderbookSummaryPublisher {
    pub fn new(watch_rx: watch::Receiver<Summary>) -> OrderbookSummaryPublisher {
        OrderbookSummaryPublisher { watch_rx }
    }

    // Spawns a process to receive OrderBookUpdates and merge them into OrderBookData which can then produce a merged view of orderbooks.
    // It will send the updated merged order book to the watch, which the clients then receive.
    pub async fn start(
        mut exchange_rx: mpsc::Receiver<OrderBookUpdate>,
        watch_tx: watch::Sender<Summary>,
    ) {
        tokio::spawn(async move {
            let mut orderbook_data = OrderBookData::default();

            loop {
                match exchange_rx.recv().await {
                    None => debug!("empty exchange update received on exchange channel"),
                    Some(orderbook_update) => {
                        let now = Instant::now();

                        let instant = orderbook_update.ts.clone();
                        let exchange = orderbook_update.exchange.clone();

                        orderbook_data.update_exchange_data(orderbook_update);

                        histogram!("orderbook_merge.time_taken_s", now.elapsed().as_secs_f64());

                        let summary = orderbook_data.summary();
                        watch_tx
                            .send(summary)
                            .expect("something went wrong publishing watch...");

                        histogram!(
                            format!("exchange.{}.time_taken_s", exchange),
                            instant.elapsed().as_secs_f64()
                        ); // Would be better if exchange was a dimension.
                    }
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct AggregatorPublisher {}

impl AggregatorPublisher {}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookSummaryPublisher {
    type BookSummaryStream = ReceiverStream<Result<Summary, tonic::Status>>;

    async fn book_summary(
        &self,
        _request: tonic::Request<crate::orderbook::Empty>,
    ) -> Result<tonic::Response<Self::BookSummaryStream>, tonic::Status> {
        let (tx, rx) = mpsc::channel(4);

        let mut wrx: watch::Receiver<Summary> = self.watch_rx.clone();

        tokio::spawn(async move {
            loop {
                match wrx.changed().await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("unexpected error waiting watch: {:?}", e);
                    }
                }

                let val = wrx.borrow().clone();

                match tx.send(Ok(val)).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("unexpected error waiting watch: {:?}", e);
                    }
                };
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}
