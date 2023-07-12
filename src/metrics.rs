#![allow(non_snake_case)]
use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use prometheus::{opts, HistogramVec, IntCounterVec, Registry};

use rocket::{
    fairing::{Fairing, Info, Kind},
    Data, Request, Response,
};

pub struct PrometheusMetrics {
    http_requests_total: IntCounterVec,
    http_requests_duration_seconds: HistogramVec,
    registry: Registry,
}

impl PrometheusMetrics {
    pub fn new(namespace: &str) -> Self {
        let registry = Registry::new();

        let http_requests_total_opts =
            opts!("http_requests_total", "Total number of HTTP requests").namespace(namespace);
        let http_requests_total =
            IntCounterVec::new(http_requests_total_opts, &["endpoint", "method", "status"])
                .unwrap();
        let http_requests_duration_seconds_opts = opts!(
            "http_requests_duration_seconds",
            "HTTP request duration in seconds for all requests"
        )
        .namespace(namespace);
        let http_requests_duration_seconds = HistogramVec::new(
            http_requests_duration_seconds_opts.into(),
            &["endpoint", "method", "status"],
        )
        .unwrap();

        registry
            .register(Box::new(http_requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(http_requests_duration_seconds.clone()))
            .unwrap();

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
    pub rw_lock: Arc<RwLock<PrometheusMetrics>>,
}

impl ArcRwLockPrometheus {
    pub fn new(prometheus: Arc<RwLock<PrometheusMetrics>>) -> Self {
        Self {
            rw_lock: prometheus,
        }
    }
}

impl Clone for ArcRwLockPrometheus {
    fn clone(&self) -> Self {
        Self {
            rw_lock: Arc::clone(&self.rw_lock),
        }
    }
}

impl ArcRwLockPrometheusTrait for ArcRwLockPrometheus {
    type ArcRwLock = Arc<RwLock<PrometheusMetrics>>;

    fn clone(&self) -> Arc<RwLock<PrometheusMetrics>> {
        Arc::clone(&self.rw_lock)
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
        self.rw_lock
            .read()
            .unwrap()
            .http_requests_total
            .with_label_values(&[endpoint, method, status.as_str()])
            .inc();

        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(duration) = start_time.0.map(|st| st.elapsed()) {
            let duration_secs = duration.as_secs_f64();
            self.rw_lock
                .read()
                .unwrap()
                .http_requests_duration_seconds
                .with_label_values(&[endpoint, method, status.as_str()])
                .observe(duration_secs);
        }
    }
}
