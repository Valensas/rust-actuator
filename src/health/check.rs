#[cfg(feature = "health-diesel")]
use ::diesel::Connection;
#[cfg(feature = "health-tonic")]
use ::tonic::transport::Channel;
#[cfg(feature = "health-tonic")]
use tonic_health::pb::health_client::HealthClient;

use super::Indicator;

pub mod liveness;
pub mod readiness;
#[cfg(feature = "health-tonic")]
pub mod tonic;

/// Liveness probe for server. Exposes `/health/liveness` route that
/// returns 200 OK when all indicators are up and 503 Service Unavailable
/// when at least one indicator is down.
pub struct Liveness {
    indicators: Vec<Box<dyn Indicator>>,
}

/// Readiness probe for server. Exposes `/health/readiness` route that
/// returns 200 OK when all indicators are up and 503 Service Unavailable
/// when at least one indicator is down.
pub struct Readiness {
    indicators: Vec<Box<dyn Indicator>>,
}

/// Health indicator for tonic-health. Uses the gRPC health protocol to
/// verify that a gRPC server is serving the given service.
#[derive(Clone)]
#[cfg(feature = "health-tonic")]
pub struct Tonic {
    /// The name of the indicator.
    name: String,
    /// The service to verify is serving.
    service: String,
    /// The client to use for health checking.
    client: HealthClient<Channel>,
}

#[cfg(feature = "health-diesel")]
pub mod diesel;

/// Health indicator for diesel. Executes a given query using a connection
/// provided by the connection provider.
#[cfg(feature = "health-diesel")]
pub struct Diesel<C: Connection, T: Fn() -> Result<C, String>>
where
    T: Sync + Send,
{
    /// The name of the indicator.
    name: String,
    /// The connection provider.
    connection_provider: T,
    /// The query to execute to verify health.
    query: String,
}
