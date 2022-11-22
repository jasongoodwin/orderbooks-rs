use std::time::Instant;

use metrics::histogram;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio_stream::wrappers::ReceiverStream;

use crate::exchange::OrderBookUpdate;
use crate::orderbook::orderbook_aggregator_server::OrderbookAggregator;
use crate::orderbook::Summary;
use crate::orderbook_data::OrderBookData;

#[derive(Debug)]
pub(crate) struct OrderbookAggregatorServer {
    pub(crate) watch_rx: watch::Receiver<Summary>,
}

impl OrderbookAggregatorServer {
    pub fn new(watch_rx: watch::Receiver<Summary>) -> OrderbookAggregatorServer {
        OrderbookAggregatorServer { watch_rx }
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorServer {
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
                tx.send(Ok(val)).await.unwrap(); // This will panic if something weird happens.
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

pub struct AggregatorProcess {
    // receives new order book data from exchanges
    // sends the updated summary to a watch for clients
    exchange_rx: mpsc::Receiver<OrderBookUpdate>,
    watch_tx: watch::Sender<Summary>,
}

impl AggregatorProcess {
    pub async fn start(
        exchange_rx: mpsc::Receiver<OrderBookUpdate>,
        watch_tx: watch::Sender<Summary>,
    ) {
        tokio::spawn(async move {
            let mut ap = AggregatorProcess {
                exchange_rx,
                watch_tx,
            };

            let mut orderbook_data = OrderBookData::default();

            loop {
                match ap.exchange_rx.recv().await {
                    None => debug!("empty exchange update received on exchange channel"),
                    Some(orderbook_update) => {
                        let now = Instant::now();

                        let instant = orderbook_update.ts.clone();
                        let exchange = orderbook_update.exchange.clone();

                        orderbook_data.update_exchange_data(orderbook_update);

                        histogram!("orderbook_merge.time_taken_s", now.elapsed().as_secs_f64());

                        let summary = orderbook_data.summary();
                        ap.watch_tx
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
