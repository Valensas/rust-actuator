use prometheus::Registry;
use valensas_actuator::{actuate, prometheus::HttpRequestCollectorConfig};

#[tokio::main]
async fn main() {
    let rocket = rocket::build();
    let registry = Registry::new();

    actuate(rocket)
        .with_metrics_endpoint(registry.clone())
        .with_rocket_metrics_collector(&HttpRequestCollectorConfig::default(registry))
        .unwrap()
        .get()
        .ignite()
        .await
        .unwrap()
        .launch()
        .await
        .unwrap();
}
