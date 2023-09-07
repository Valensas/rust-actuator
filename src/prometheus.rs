use ::prometheus::HistogramVec;
use prometheus::{opts, HistogramOpts, Registry};

use crate::Actuator;

pub mod endpoint;
pub mod http_request_collector_metrics;
pub mod rocket_http_collector;

#[cfg(feature = "prometheus-tonic")]
pub mod tonic_grpc_collector;

#[cfg(feature = "prometheus-r2d2")]
pub mod r2d2;

pub struct Endpoint {
    registry: Registry,
}

pub struct HttpRequestCollectorConfig {
    registry: Registry,
    http_requests_duration_options: HistogramOpts,
}

impl HttpRequestCollectorConfig {
    pub fn default(registry: Registry) -> Self {
        Self {
            registry,
            http_requests_duration_options: opts!(
                "http_requests_duration_seconds",
                "HTTP request duration in seconds for all requests"
            )
            .into(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct HttpRequestCollectorMetrics {
    http_requests_duration_seconds: HistogramVec,
}

pub struct RocketHttpCollector {
    metrics: HttpRequestCollectorMetrics,
}

#[derive(Clone)]
#[cfg(feature = "prometheus-tonic")]
pub struct TonicGrpcCollectorLayer {
    metrics: HttpRequestCollectorMetrics,
}

impl Actuator {
    pub fn with_metrics_endpoint(self, registry: ::prometheus::Registry) -> Actuator {
        self.with_configurer(Endpoint::new(registry))
    }

    pub fn with_rocket_metrics_collector(
        self,
        config: &HttpRequestCollectorConfig,
    ) -> Result<Actuator, prometheus::Error> {
        Ok(self.with_configurer(RocketHttpCollector::new(config)?))
    }
}
