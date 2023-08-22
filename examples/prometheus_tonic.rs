use prometheus::Registry;
use valensas_actuator::{actuate, prometheus::HttpRequestCollectorConfig};

#[tokio::main]
async fn main() {
    let rocket = rocket::build();
    let registry = Registry::new();

    let (_, health_service) = tonic_health::server::health_reporter();
    let config = HttpRequestCollectorConfig::default(registry.clone());
    let layer = tower::ServiceBuilder::new()
        .layer(valensas_actuator::prometheus::TonicGrpcCollectorLayer::new(&config).unwrap())
        .into_inner();

    tokio::spawn(
        tonic::transport::Server::builder()
            .layer(layer)
            .add_service(health_service)
            .serve("127.0.0.1:50051".parse().unwrap()),
    );

    actuate(rocket)
        .with_metrics_endpoint(registry.clone())
        .get()
        .ignite()
        .await
        .unwrap()
        .launch()
        .await
        .unwrap();
}
