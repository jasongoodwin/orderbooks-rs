use crate::orderbook::orderbook_aggregator_server::*;
use crate::orderbook::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Debug)]
pub struct OrderbookAggregatorServer;

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

        //
        // let output = async_stream::try_stream! {
        //     // let asks = Level {
        //     //     exchange: "yo".into(),
        //     //     amount: vec![],
        //     //     price: vec![],
        //     // };
        //     println!("here");
        //     loop {
        //         println!("in stream");
        //         tx.send(Ok(Summary {
        //         asks: vec![],
        //         bids: vec![],
        //         spread: 0.0,
        //     })).await.unwrap();
        //
        //     }
        // };

        let txr = Arc::new(tx.clone());

        tokio::spawn(async move {
            loop {
                println!("sending");
                txr.clone()
                    .send(Ok(Summary {
                        asks: vec![],
                        bids: vec![],
                        spread: 0.0,
                    }))
                    .await
                    .unwrap();
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}
