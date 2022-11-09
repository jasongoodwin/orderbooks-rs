//! IndraDB: a graph datastore.
//!
//! IndraDB is broken up into a library and an application. This is the
//! library, which you would use if you want to create new datastore
//! implementations, or plug into the low-level details of IndraDB. For most
//! use cases, you can use the application, which exposes an API and scripting
//! layer.

use metrics::{counter, gauge, histogram, increment_counter, register_counter, register_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;

// Gauge to show how many services are running. The server will start and increment the gauge by 1.
// This can be used for alerting if services die.
const RUNNING_GAUGE: &str = "running";

// starts the prometheus exporter and registers all metrics.
// metrics can be seen at localhost:9000
pub fn register_all() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install metric recorder/exporter");

    gauge!(RUNNING_GAUGE, 1.0);
}
