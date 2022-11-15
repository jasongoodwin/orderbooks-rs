use crate::orderbook::orderbook_aggregator_server::*;
use crate::orderbook::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::sync::watch::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;
use crate::orderbook::Summary;

#[derive(Debug)]
pub struct OrderbookAggregatorServer {
    pub(crate) watch_rx: watch::Receiver<Summary>,
    last_summary: Summary,
    // subscribers: Vec<Receiver<Result<Summary, tonic::Status>>>
}

impl OrderbookAggregatorServer {
    pub fn new(watch_rx: watch::Receiver<Summary>) -> OrderbookAggregatorServer {
        OrderbookAggregatorServer {
            watch_rx,
            last_summary: Summary{
                spread: 0.0,
                bids: vec![],
                asks: vec![]
            },
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
