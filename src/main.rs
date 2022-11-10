//! orderbooks-rs: glean some insight from market depth!
//!
//! The application tries to be simple and flat as best it can.
//! The application is separated into modules for major functionality.
//! See the Settings.toml file for configuration.

// use log::LevelFilter;

// use std::collections::HashMap;
#[macro_use]
extern crate log;
mod app_config;
mod metrics;
mod result;

#[tokio::main]
async fn main() {
    env_logger::init();

    let conf = app_config::AppConfig::new().expect("couldn't read Settings...");

    // let pairs: Vec<String> = conf.

    info!(
        "Starting app with configured pairs: {:?}",
        conf.spot_pairs()
    );
    info!("feel free to adjust trading pairs in [Settings.toml]!");

    metrics::register_all();

    // let exchanges: Vec<String> = settings.get("exchanges").expect("error in exchange config...");
    // info!("starting all stream inputs for pairs on exchanges: {:?}", exchanges);

    loop {}
}
