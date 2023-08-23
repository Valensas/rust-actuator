use std::time::Instant;

use rocket::{
    fairing::{Fairing, Info, Kind},
    Build, Data, Request, Response, Rocket,
};

use crate::actuator::RocketConfigurerer;

use super::{HttpRequestCollectorConfig, HttpRequestCollectorMetrics, RocketHttpCollector};

impl RocketHttpCollector {
    pub fn new(config: &HttpRequestCollectorConfig) -> Result<Self, prometheus::Error> {
        Ok(Self {
            metrics: HttpRequestCollectorMetrics::from(config)?,
        })
    }
}

#[async_trait::async_trait]
impl RocketConfigurerer for RocketHttpCollector {
    fn configure(self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket.attach(RocketHttpCollectorFairing {
            metrics: self.metrics.clone(),
        })
    }
}

struct RocketHttpCollectorFairing {
    metrics: HttpRequestCollectorMetrics,
}

#[async_trait::async_trait]
impl Fairing for RocketHttpCollectorFairing {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus http metric collector",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        req.local_cache(|| Some(Instant::now()));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, response: &mut Response<'r>) {
        let path = req.route().map(|r| r.uri.as_str()).unwrap_or("UNKNOWN");

        let method = req.method().as_str();
        let status = response.status().code.to_string();

        let request_time = match req.local_cache(|| None as Option<Instant>) {
            Some(start_time) => start_time.elapsed(),
            None => {
                log::error!(
                    "cannot register prometheu metrics: unable to find start time for request"
                );
                return;
            }
        };

        self.metrics
            .http_requests_duration_seconds
            .with_label_values(&[path, method, status.as_str()])
            .observe(request_time.as_secs_f64());
    }
}
