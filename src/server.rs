//! orderbooks-rs: glean some insight from market depth!
//!
//! The application tries to be simple and flat as best it can.
//! The application is separated into modules for major functionality.
//! See the Settings.toml file for configuration.

use std::fmt::format;
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

use crate::exchange::OrderBookUpdate;
use crate::exchange_service::{AggregatorProcess, OrderbookAggregatorServer};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
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
    let exchange_configs = conf
        .exchange_configs()
        .expect("no exchange configs exist...");

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

    let (tx, mut rx) = mpsc::channel(32);

    // Start the process that aggregates order books and supplies updates to the watch for single producer multi consumer semantics.
    AggregatorProcess::start(rx, watch_tx).await;

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

        exchange::spawn_new_ws(conf.clone(), tx.clone()).await; // todo await here unnecessary
                                                                // .expect(&*format!("couldn't start exchange stream for {:?}", conf.clone()));
    }

    let addr = "[::1]:10000".parse().unwrap();
    let route_guide = OrderbookAggregatorServer::new(watch_rx);
    let svc =
        crate::orderbook::orderbook_aggregator_server::OrderbookAggregatorServer::new(route_guide);
    Server::builder().add_service(svc).serve(addr).await?;

    println!("GRPC Server stopped. shutting down...");
    Ok(())
}
