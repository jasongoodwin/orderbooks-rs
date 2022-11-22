//! orderbook-rs server - this is the main entry point to the server application.
#[macro_use]
extern crate log;

use tokio::sync::{mpsc, watch};
use tonic::transport::Server;

use crate::orderbook::*;
use crate::orderbook_aggregator::OrderbookSummaryPublisher;

mod app_config;
mod exchange;
mod metrics;
mod orderbook_aggregator;
mod orderbook_data;
mod result;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[tokio::main]
async fn main() -> result::Result<()> {
    println!("Starting app... Use RUST_LOG=info to enable more logging.");
    env_logger::init();

    let conf = app_config::AppConfig::new().expect("couldn't read Settings...");

    let spot_pair = conf.spot_pair().unwrap();
    let enabled_exchanges = conf.enabled_exchanges().unwrap();
    let exchange_configs = conf
        .exchange_configs()
        .expect("no exchange configs exist...");

    println!(
        "\nConfigured pair: {:?}.\nEnabled exchanges: {:?}.\n",
        &spot_pair, &enabled_exchanges
    );

    metrics::start_server_and_register();

    // watch is used to send messages to the server/client connections.
    // Each client connection will observe when there is an update and then will read the most current values.
    // Note that borrows of the value will hold a read lock so they should be very short lived.
    // This shouldn't cause any contention, but may need to revisit the use of channels
    // Optimization: use eg Arc<Summary> instead of Summary for the channels to prevent cloning of the Summary per client.
    let (watch_tx, watch_rx) = watch::channel(Summary {
        spread: 0.0,
        bids: vec![],
        asks: vec![],
    });

    let (tx, rx) = mpsc::channel(32);

    // Start the process that aggregates order books and supplies updates to the watch for single producer multi consumer semantics.
    OrderbookSummaryPublisher::start(rx, watch_tx).await;

    for exchange in enabled_exchanges {
        let conf = exchange_configs
            .iter()
            .find(|ex| ex.id == exchange)
            .expect(&*format!(
                "no config for configured exchange {}",
                exchange.clone()
            ))
            .clone();

        info!("starting exchange stream for: [{}]", exchange);

        exchange::create_exchange_ws_connection(conf.clone(), tx.clone());
    }

    let addr = "[::1]:10000".parse().unwrap();
    let route_guide = OrderbookSummaryPublisher::new(watch_rx);
    let svc =
        crate::orderbook::orderbook_aggregator_server::OrderbookAggregatorServer::new(route_guide);
    Server::builder().add_service(svc).serve(addr).await?;

    println!("GRPC Server stopped. shutting down...");
    Ok(())
}
