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
use tonic::transport::Server;

pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[tokio::main]
async fn main() -> result::Result<()> {
    println!("Starting app... Use RUST_LOG=info to enable logging.");
    env_logger::init();

    let conf = app_config::AppConfig::new().expect("couldn't read Settings...");

    let spot_pairs = conf.spot_pairs().unwrap();
    let enabled_exchanges = conf.enabled_exchanges().unwrap();

    println!(
        "\nConfigured pairs: {:?}.\nEnabled exchanges: {:?}.\n",
        &spot_pairs, &enabled_exchanges
    );

    metrics::register_all();

    // let exchanges: Vec<String> = settings.get("exchanges").expect("error in exchange config...");
    // info!("starting all stream inputs for pairs on exchanges: {:?}", exchanges);

    let addr = "[::1]:10000".parse().unwrap();

    let route_guide = exchange_service::OrderbookAggregatorServer {
        // features: Arc::new(data::load()),
    };

    let svc =
        crate::orderbook::orderbook_aggregator_server::OrderbookAggregatorServer::new(route_guide);

    Server::builder().add_service(svc).serve(addr).await?;
    // let route_guide = exchange_service::OrderbookAggregator {
    // features: Arc::new(data::load()),
    // };

    // let svc = OrderbookAggregator::new(route_guide);

    // Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
