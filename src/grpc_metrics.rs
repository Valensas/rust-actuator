use std::{sync::{Arc}, time::Instant};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use rocket::http::hyper;
use rocket::http::hyper::Body;
use rocket::log;
use rocket::tokio::sync::RwLock;
use tonic::body::BoxBody;
use tower::{Layer, Service};

use crate::metrics::PrometheusMetrics;

#[derive(Clone)]
pub struct GrpcMetricLayer {
    prometheus: Arc<RwLock<PrometheusMetrics>>,
}

impl GrpcMetricLayer {
    pub fn new(prom: Arc<RwLock<PrometheusMetrics>>) -> Self {
        Self {
            prometheus: prom
        }
    }
}

impl<S> Layer<S> for GrpcMetricLayer {
    type Service = GrpcMetric<S>;

    fn layer(&self, service: S) -> Self::Service {
        GrpcMetric { inner: service, prometheus: Arc::clone(&self.prometheus) }
    }
}

#[derive(Clone)]
pub struct GrpcMetric<S> {
    inner: S,
    prometheus: Arc<RwLock<PrometheusMetrics>>,
}

type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output=T> + Send>>;

impl<S> Service<hyper::Request<Body>> for GrpcMetric<S>
    where
        S: Service<hyper::Request<Body>, Response=hyper::Response<BoxBody>> + Clone + Send + 'static,
        S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let arc_prometheus_metrics = self.prometheus.clone();
        Box::pin(async move {
            let prometheus_metrics = arc_prometheus_metrics.read().await;

            let start = Instant::now();

            let grpc_method = req.uri().path().to_string();
            let response = inner.call(req).await?;

            let grpc_status = match response.headers().get("grpc-status") {
                Some(header) => header.to_str().unwrap().to_string(),
                None => String::from("Unknown").to_string(),
            };

            let duration = Instant::now().duration_since(start);

            log::debug_!(
                "Tracing Record: {} is called. Response time is {}. Status code: {}. Grpc Status Code: {}",
                grpc_method,
                format!("{}s {}ms {}ns", duration.as_secs(), duration.subsec_millis(), duration.subsec_nanos()),
                response.status(),
                grpc_status
            );

            prometheus_metrics.http_requests_duration_seconds()
                .with_label_values(&[grpc_method.as_str(), "grpc", grpc_status.as_str()])
                .observe(duration.as_secs_f64());

            prometheus_metrics.http_requests_total()
                .with_label_values(&[grpc_method.as_str(), "grpc", grpc_status.as_str()])
                .inc();

            Ok(response)
        })
    }
}
