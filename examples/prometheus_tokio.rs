use prometheus::Registry;
use valensas_actuator::actuate;

#[tokio::main]
async fn main() {
    let rocket = rocket::build();
    let registry = Registry::new();

    let runtime_metric_collector_config =
        valensas_actuator::prometheus::tokio::RuntimeMetricCollectorConfig::default(
            tokio::runtime::Handle::current(),
        );
    let runtime_metric_collector =
        valensas_actuator::prometheus::tokio::RuntimeMetricCollector::new(
            runtime_metric_collector_config,
        )
        .unwrap();

    registry
        .register(Box::new(runtime_metric_collector))
        .unwrap();

    actuate(rocket)
        .with_metrics_endpoint(registry)
        .get()
        .ignite()
        .await
        .unwrap()
        .launch()
        .await
        .unwrap();
}
