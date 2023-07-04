//! # Rocket Prometheus Metrics Library
//!
//! This library provides Prometheus metrics collection capabilities for Rocket applications.
//! It allows you to track and record metrics related to HTTP requests made to Rocket endpoints.
//!
//! ## Usage
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
//! use rocket_metric_collector::metrics::{ArcRwLockPrometheus, PrometheusMetrics};
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
//! ## Example
//!
//! Here's an example of accessing the metrics from the `ArcRwLockPrometheus` instance:
//!
//! ```rust
//! use rocket::{get};
//! use rocket_metric_collector::metrics::metrics::{PrometheusMetrics, ArcRwLockPrometheus};
//!
//! #[get("/hello")]
//! async fn hello() -> &'static str {
//!     // Increment the total number of HTTP requests for the '/hello' endpoint
//!     prometheus.http_requests_total()
//!         .with_label_values(&["/hello", "GET", "200"])
//!         .inc();
//!
//!     "Hello, World!"
//! }
//! ```
//!
//! In the above example, the `http_requests_total` counter is incremented for the `/hello` endpoint with the method `GET` and status `200`.
//!
//! ---
//!
//! With this library, you can easily collect Prometheus metrics for your Rocket application endpoints and gain insights into your application's performance.
//!



#[allow(non_snake_case)]
use std::{ time::Instant, sync::{ RwLock, Arc } };

use prometheus::{ opts, HistogramVec, IntCounterVec, Registry };

use rocket::{ fairing::{ Info, Kind, Fairing }, Request, Data, Response };

pub struct PrometheusMetrics {
    http_requests_total: IntCounterVec,
    http_requests_duration_seconds: HistogramVec,
    registry: Registry,
}

impl PrometheusMetrics {
    pub fn new(namespace: &str) -> Self {
        let registry = Registry::new();

        let http_requests_total_opts = opts!(
            "http_requests_total",
            "Total number of HTTP requests"
        ).namespace(namespace);
        let http_requests_total = IntCounterVec::new(
            http_requests_total_opts,
            &["endpoint", "method", "status"]
        ).unwrap();
        let http_requests_duration_seconds_opts = opts!(
            "http_requests_duration_seconds",
            "HTTP request duration in seconds for all requests"
        ).namespace(namespace);
        let http_requests_duration_seconds = HistogramVec::new(
            http_requests_duration_seconds_opts.into(),
            &["endpoint", "method", "status"]
        ).unwrap();

        registry.register(Box::new(http_requests_total.clone())).unwrap();
        registry.register(Box::new(http_requests_duration_seconds.clone())).unwrap();

        Self {
            http_requests_total,
            http_requests_duration_seconds,
            registry,
        }
    }

    pub const fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn http_requests_total(&self) -> &IntCounterVec {
        &self.http_requests_total
    }

    pub fn http_requests_duration_seconds(&self) -> &HistogramVec {
        &self.http_requests_duration_seconds
    }
}

impl Clone for PrometheusMetrics {
    fn clone(&self) -> Self {
        Self {
            http_requests_total: self.http_requests_total.clone(),
            http_requests_duration_seconds: self.http_requests_duration_seconds.clone(),
            registry: self.registry.clone(),
        }
    }
}

#[derive(Copy, Clone)]
struct TimerStart(Option<Instant>);

pub trait ArcRwLockPrometheusTrait {
    type ArcRwLock;
    fn clone(&self) -> Arc<RwLock<PrometheusMetrics>>;
}

pub struct ArcRwLockPrometheus {
    pub rwLock: Arc<RwLock<PrometheusMetrics>>,
}

impl ArcRwLockPrometheus {
    pub fn new(prometheus: Arc<RwLock<PrometheusMetrics>>) -> Self {
        Self {
            rwLock: prometheus,
        }
    }
}

impl Clone for ArcRwLockPrometheus {
    fn clone(&self) -> Self {
        Self {
            rwLock: Arc::clone(&self.rwLock),
        }
    }
}

impl ArcRwLockPrometheusTrait for ArcRwLockPrometheus {
    type ArcRwLock = Arc<RwLock<PrometheusMetrics>>;

    fn clone(&self) -> Arc<RwLock<PrometheusMetrics>> {
        Arc::clone(&self.rwLock)
    }
}

#[rocket::async_trait]
impl Fairing for ArcRwLockPrometheus {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus metric collection",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        req.local_cache(|| TimerStart(Some(Instant::now())));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, response: &mut Response<'r>) {
        if req.route().is_none() {
            return;
        }

        let endpoint = req.route().unwrap().uri.as_str();
        let method = req.method().as_str();
        let status = response.status().code.to_string();
        self.rwLock
            .read()
            .unwrap()
            .http_requests_total.with_label_values(&[endpoint, method, status.as_str()])
            .inc();

        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(duration) = start_time.0.map(|st| st.elapsed()) {
            let duration_secs = duration.as_secs_f64();
            self.rwLock
                .read()
                .unwrap()
                .http_requests_duration_seconds.with_label_values(
                    &[endpoint, method, status.as_str()]
                )
                .observe(duration_secs);
        }
    }
}
