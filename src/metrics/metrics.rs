#![allow(non_snake_case)]
use std::{ time::Instant, sync::{ RwLock, Arc } };

use prometheus::{ opts, HistogramVec, IntCounterVec, Registry };

use rocket::{ fairing::{ Info, Kind, Fairing }, Request, Data, Response };

pub struct PrometheusMetrics {
    http_requests_total: IntCounterVec,
    http_requests_duration_seconds: HistogramVec,
    registery: Registry,
}

impl PrometheusMetrics {
    pub fn new(namespace: &str) -> Self {
        let registery = Registry::new();

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

        registery.register(Box::new(http_requests_total.clone())).unwrap();
        registery.register(Box::new(http_requests_duration_seconds.clone())).unwrap();

        Self {
            http_requests_total,
            http_requests_duration_seconds,
            registery,
        }
    }

    pub const fn registry(&self) -> &Registry {
        &self.registery
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
            registery: self.registery.clone(),
        }
    }
}

#[derive(Copy, Clone)]
struct TimerStart(Option<Instant>);

pub trait ArcRwLockPrometheusTrait {
    type MyRwLock;
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
    type MyRwLock = Arc<RwLock<PrometheusMetrics>>;

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
