//! Sets up metrics + exporter
use metrics::gauge;
use metrics_exporter_prometheus::PrometheusBuilder;

// Gauge to show how many services are running. The server will start and increment the gauge by 1.
// This can be used for alerting if services die (eg gauge = 0 means nothing running!)
const RUNNING_GAUGE: &str = "running";

// starts the prometheus exporter and registers the service.
// metrics can be seen at localhost:9000
pub fn start_server_and_register() {
    info!("starting metrics server @ 0.0.0.0:9000");

    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install metric recorder/exporter");

    gauge!(RUNNING_GAUGE, 1.0);
}
