//! orderbooks-rs: glean some insight from market depth!
//!
//! The application tries to be simple and flat as best it can.
//! The application is separated into modules for major functionality.
//! See the Settings.toml file for configuration.

// use log::LevelFilter;
use crate::orderbook::orderbook_aggregator_server::*;
use crate::orderbook::*;
// use std::collections::HashMap;
#[macro_use]
extern crate log;
mod app_config;
mod exchange;
mod exchange_service;
mod metrics;
mod result;

use crate::exchange_service::OrderbookAggregatorServer;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Server;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[tokio::main]
async fn main() -> result::Result<()> {
    println!("Starting app... Use RUST_LOG=info to enable logging.");
    env_logger::init();

    let conf = app_config::AppConfig::new().expect("couldn't read Settings...");

    let spot_pair = conf.spot_pair().unwrap();
    let enabled_exchanges = conf.enabled_exchanges().unwrap();

    println!(
        "\nConfigured pair: {:?}.\nEnabled exchanges: {:?}.\n",
        &spot_pair, &enabled_exchanges
    );

    metrics::register_all();

    // let exchanges: Vec<String> = settings.get("exchanges").expect("error in exchange config...");
    // info!("starting all stream inputs for pairs on exchanges: {:?}", exchanges);

    // watch is used to send messages to the server/client connections.
    // Each client connection will observe when there is an update and then will read the most current values.
    // Note that borrows of the value will hold a read lock so they should be very short lived.
    // This shouldn't cause any contention, but may need to revisit the use of channels
    // Optimization: use eg Arc<Summary> instead of Summary for the channels to prevent cloning of the Summary per client.
    let (watch_tx, mut watch_rx) = watch::channel(Summary {
        spread: 0.0,
        bids: vec![],
        asks: vec![],
    });

    tokio::spawn(async move {
        // TEST LOOP! shows that you can send updates to the watch and client will get 'em.
        let mut i = 0.0;
        loop {
            println!("sending update");
            i = i + 0.1;
            watch_tx
                .send(Summary {
                    spread: i,
                    bids: vec![],
                    asks: vec![],
                })
                .expect("uh oh");
            use tokio::time::{sleep, Duration};
            sleep(Duration::from_millis(100)).await;
        }
        // tx.broadcast("goodbye").unwrap();
    });

    let addr = "[::1]:10000".parse().unwrap();
    let route_guide = OrderbookAggregatorServer::new(watch_rx);
    let svc =
        crate::orderbook::orderbook_aggregator_server::OrderbookAggregatorServer::new(route_guide);
    Server::builder().add_service(svc).serve(addr).await?;

    println!("GRPC Server stopped. shutting down...");
    Ok(())
}
