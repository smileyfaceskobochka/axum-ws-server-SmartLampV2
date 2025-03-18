// metrics/mod.rs
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

pub fn setup_metrics(port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()
        .expect("Failed to setup metrics");
}

pub fn encode() -> String {
    // Используйте билдер для получения handle
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install recorder");
    handle.render()
}