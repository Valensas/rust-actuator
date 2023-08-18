//! # Rocket Prometheus Metrics Library - Valensas Actuator
//!
//! This library provides Prometheus metrics collection capabilities for Rocket applications.
//! It allows you to track and record metrics related to HTTP requests made to Rocket endpoints.
//!
//! ## Installation
//! Add the following to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! valensas_actuator = "0.2.0"
//! ```
//!
//! ## Enabling gRPC Metrics
//! To enable gRPC metrics, you need to add the `grpc` feature to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! valensas-actuator = { version = "0.2.0", features = ["grpc"] }
//! ```
//! Once you have enabled the grpc feature, you can use the GrpcMetricLayer to collect gRPC metrics.
//! The GrpcMetricLayer will automatically collect metrics for gRPC requests handled by the service.
//!
//! ## Usage
//!
//! `Configure Rocket server`
//!
//! To use this library, you need to create an instance of `ArcRwLockPrometheus` and attach it as a Rocket fairing.
//! The fairing will automatically collect metrics for each incoming HTTP request and response.
//!
//!
//! Create an instance of `ArcRwLockPrometheus` and attach it as a fairing to your Rocket application:
//!
//! ```rust
//! #[macro_use]
//! extern crate rocket;
//!
//! use std::sync::{Arc, RwLock};
//! use rocket::{Build, Rocket};
//! use valensas_actuator::metrics::{ArcRwLockPrometheus, PrometheusMetrics};
//!
//! #[get("/")]
//! fn index() -> &'static str {
//!     "Hello, world!"
//! }
//!
//! #[launch]
//! fn rocket() -> Rocket<Build> {
//!     let prometheus = Arc::new(RwLock::new(PrometheusMetrics::new("your_namespace")));
//!     let prometheus_fairing = ArcRwLockPrometheus::new(prometheus.clone());
//!
//!     Rocket::build()
//!         .attach(prometheus_fairing.clone())
//!         .manage(prometheus_fairing)
//!         .mount("/", routes![index])
//! }
//! ```
//!
//! Make sure to replace `"your_namespace"` with your desired namespace for Prometheus metrics.
//!
//! In the above example, `PrometheusMetrics::new("your_namespace")` creates a new instance of `PrometheusMetrics` with the specified namespace.
//! The `ArcRwLockPrometheus` instance is then cloned and passed to the Rocket application as a managed state and as a fairing using `rocket.manage()` and `.attach()` methods respectively.
//!
//! With the fairing attached, the library will automatically collect metrics for each incoming request and response.
//! The collected metrics can be accessed through the `ArcRwLockPrometheus` instance.
//!
//! `Configure Grpc Metrics`
//!
//! To use the GrpcMetricLayer, you need to create an instance of it and add it to your Rocket service.
//!
//! ```rust
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! use tokio::sync::RwLock;
//! use tonic::transport::Server;
//! use valensas_actuator::metrics::PrometheusMetrics;
//!
//! let prometheus = Arc::new(RwLock::new(PrometheusMetrics::new("your_namespace")));
//! let layer = tower::ServiceBuilder::new()
//!     .timeout(Duration::from_secs(30))
//!     .layer(GrpcMetricLayer::new(Arc::clone(&prometheus)))
//!     .into_inner();
//!
//! tokio::spawn(
//!     Server::builder()
//!         .layer(layer)
//!         .add_service(YOUR_SERVICE)
//!         .serve(address)
//! );
//! ```
//!
//! ## Example
//!
//! Here's an example of accessing the metrics from the `ArcRwLockPrometheus` instance:
//!
//! ```rust
//! use std::sync::{Arc, RwLock};
//! use rocket::{Build, Rocket, State, routes, get, launch};
//! use prometheus::{Encoder, TextEncoder};
//! use valensas_actuator::metrics::{ArcRwLockPrometheus, PrometheusMetrics};
//!
//! #[launch]
//! fn rocket() -> Rocket<Build> {
//!     let prometheus = Arc::new(RwLock::new(PrometheusMetrics::new("your_namespace")));
//!     let prometheus_fairing = ArcRwLockPrometheus::new(prometheus.clone());
//!
//!     Rocket::build()
//!         .attach(prometheus_fairing.clone())
//!         .manage(prometheus_fairing)
//!         .mount("/", routes![index, metrics])
//! }
//!
//! #[get("/metrics")]
//! async fn metrics(
//!     prometheus_metrics: &State<ArcRwLockPrometheus>
//! ) -> Result<String, rocket::response::status::Custom<String>> {
//!     let mut buffer = vec![];
//!     let encoder = TextEncoder::new();
//!     encoder
//!         .encode(&prometheus_metrics.rw_lock.read().unwrap().registry().gather(), &mut buffer)
//!         .unwrap();
//!     let body = String::from_utf8(buffer.clone()).unwrap();
//!     Ok(body)
//! }
//! ```
//!
//! In the above example, the `/metrics` endpoint returns the collected metrics of the Rocket application.
//!
//! ---
//!
//! With this library, you can easily collect Prometheus metrics for your Rocket application endpoints and gain insights into your application's performance.
//!

pub mod metrics;

#[cfg(feature = "grpc")]
pub mod grpc_metrics;

