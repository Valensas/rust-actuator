 # Valensas Actuator

 This library provides facilities for web server lifecycle operations.

 Features:

 - Health checks: liveness and readiness

   - [Diesel](https://github.com/diesel-rs/diesel) health

   - [Tonic Health](https://github.com/hyperium/tonic/tree/master/tonic-health)

   - Customizable indicators

 - [Prometheus](http://prometheus.io) metric collection

   - Rocket http request metrics

   - Tonic grpc request metrics


 ## Installation

 Add the following to your `Cargo.toml`:

 ```toml
 [dependencies]
 valensas-actuator = "*"
 ```

 ## Features

 The following crate features are available to use:

 health: includes health check related functionalities

 health-tonic: includes tonic-health health indicator

 health-diesel: includes diesel health indicator

 promtheteus-rocket: includes Prometheus scrap endpoint and Rocket http request metric collection

 prometheus-tonic: includes Tonic grpc request metric collection

 ## Examples

 For detailed usage examples, see the examples directory.


 ### health.rs

 Contains examples on how to configure health check endpoints and custom health indicators.

 Run with `cargo run --example health --features health,health-diesel`.

 ### prometheus.rs

 Contains examples on how to configure Prometheus scrap endpoint and Rocket request metric collection.

 Run with `cargo run --example prometheus --features prometheus-rocket`.

 ### prometheus_tonic.rs

 Contains examples on how to configure Prometheus scrap endpoint and Tonic gRPC request metric collection.

 Run with `cargo run --example prometheus_tonic --features prometheus-tonic`.
