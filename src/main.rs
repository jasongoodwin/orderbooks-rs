//! orderbooks-rs: glean some insight from market depth!
//!
//! The application tries to be simple and flat as best it can.
//! The application is separated into modules for major functionality.
//! See the Settings.toml file for configuration.

use log::LevelFilter;
use std::collections::HashMap;

mod metrics;

fn main() {
    let settings: config::Config = config::Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("Settings"))
        .build()
        .expect("couldn't read Settings...");

    metrics::register_all();

    loop {}

    //
    // let log_level = if settings.get::<bool>("debug")
    //     .expect("no log level in Settings found...")
    //     == true {
    //     LevelFilter::Info;
    // } else {
    //     LevelFilter::Info;
    // }

    // Print out our settings (as a HashMap)
    // println!(
    //     "{:?}",
    //
    // );
    //
    // println!("Hello, world!");
}
