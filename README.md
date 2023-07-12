# Valensas Actuator - Rocket Prometheus Metrics Library

Valensas Actuator is a Rust library that provides Prometheus metrics collection capabilities for Rocket applications. It allows you to track and record metrics related to HTTP requests made to Rocket endpoints.

## Usage

To use this library, you need to create an instance of `ArcRwLockPrometheus` and attach it as a fairing to your Rocket application. The fairing will automatically collect metrics for each incoming HTTP request and response.

For detailed information visit: https://docs.rs/valensas-actuator 

### Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
valensas-actuator = "0.1.2"
