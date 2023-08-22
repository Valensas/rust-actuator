use prometheus::HistogramVec;

use super::{HttpRequestCollectorConfig, HttpRequestCollectorMetrics};

impl HttpRequestCollectorMetrics {
    pub(crate) fn from(config: &HttpRequestCollectorConfig) -> Result<Self, prometheus::Error> {
        Ok(Self {
            http_requests_duration_seconds: Self::http_requests_duration_seconds(config)?,
        })
    }

    fn http_requests_duration_seconds(
        config: &HttpRequestCollectorConfig,
    ) -> Result<HistogramVec, prometheus::Error> {
        let histogram = HistogramVec::new(
            config.http_requests_duration_options.clone(),
            &["path", "method", "status"],
        )?;

        let res = config.registry.register(Box::new(histogram.clone()));
        if let Err(err) = res {
            match err {
                // The metric is already registered, should not be an issue
                prometheus::Error::AlreadyReg => return Ok(histogram),
                _ => {
                    return Err(err);
                }
            }
        }

        Ok(histogram)
    }
}
