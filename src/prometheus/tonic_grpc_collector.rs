use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use rocket::http::hyper::Body;
use rocket::http::hyper::{self};
use tonic::body::BoxBody;
use tower::Service;

use super::{HttpRequestCollectorConfig, HttpRequestCollectorMetrics, TonicGrpcCollectorLayer};

impl TonicGrpcCollectorLayer {
    pub fn new(config: &HttpRequestCollectorConfig) -> Result<Self, prometheus::Error> {
        Ok(Self {
            metrics: HttpRequestCollectorMetrics::from(config)?,
        })
    }
}

impl<S> tower::Layer<S> for TonicGrpcCollectorLayer {
    type Service = TonicGrpcCollector<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TonicGrpcCollector {
            inner,
            metrics: self.metrics.clone(),
        }
    }
}

#[derive(Clone)]
pub struct TonicGrpcCollector<S> {
    inner: S,
    metrics: HttpRequestCollectorMetrics,
}

type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;

impl<S> Service<hyper::Request<Body>> for TonicGrpcCollector<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
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
        let metrics = self.metrics.clone();
        Box::pin(async move {
            let start = Instant::now();

            let grpc_method = req.uri().path().to_string();
            let response = inner.call(req).await?;

            let grpc_status = match response.headers().get("grpc-status") {
                Some(header) => header.to_str().unwrap().to_string(),
                None => String::from("0").to_string(),
            };

            let duration = Instant::now().duration_since(start);

            log::debug!(
                "Tracing Record: {} is called. Response time is {}. Status code: {}. Grpc Status Code: {}",
                grpc_method,
                format!("{}s {}ms {}ns", duration.as_secs(), duration.subsec_millis(), duration.subsec_nanos()),
                response.status(),
                grpc_status
            );

            metrics
                .http_requests_duration_seconds
                .with_label_values(&[grpc_method.as_str(), "grpc", grpc_status.as_str()])
                .observe(duration.as_secs_f64());

            Ok(response)
        })
    }
}
