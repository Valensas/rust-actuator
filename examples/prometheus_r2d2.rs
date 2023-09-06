use diesel::{r2d2::ConnectionManager, SqliteConnection};
use prometheus::Registry;
use r2d2::Pool;
use valensas_actuator::actuate;

#[tokio::main]
async fn main() {
    let manager = ConnectionManager::<SqliteConnection>::new("test.db");
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .unwrap();

    let rocket = rocket::build();
    let registry = Registry::new();

    let pool_metric_collector_config =
        valensas_actuator::prometheus::r2d2::PoolMetricCollectorConfig::default("default_pool");
    let pool_metric_collector = valensas_actuator::prometheus::r2d2::PoolMetricCollector::new(
        pool,
        pool_metric_collector_config,
    )
    .unwrap();

    registry.register(Box::new(pool_metric_collector)).unwrap();

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
