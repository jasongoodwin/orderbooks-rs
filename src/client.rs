pub mod orderbook {
    tonic::include_proto!("orderbook");
}
use crate::orderbook::orderbook_aggregator_client::*;
use crate::orderbook::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:10000").await?;
    let mut stream = client
        .book_summary(tonic::Request::new(Empty {}))
        .await?
        .into_inner();

    println!("here");

    while let Some(feature) = stream.message().await? {
        println!("got it");
        println!("NOTE = {:?}", feature);
    }

    Ok(())
}
