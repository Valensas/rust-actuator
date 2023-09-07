use ::tonic::transport::Channel;
use tonic_health::pb::{
    health_check_response::ServingStatus, health_client::HealthClient, HealthCheckRequest,
};

impl crate::health::check::Tonic {
    pub fn new(name: String, service: String, client: HealthClient<Channel>) -> Self {
        Self {
            name,
            service,
            client,
        }
    }
}

#[async_trait::async_trait]
impl crate::health::Indicator for crate::health::check::Tonic {
    fn name(&self) -> &str {
        &self.name.as_str()
    }

    async fn check(&self) -> Result<(), String> {
        let response = self
            .client
            .clone()
            .check(HealthCheckRequest {
                service: self.service.clone(),
            })
            .await
            .map_err(|e| format!("status: {}, message: {}", e.code(), e.message()))?;

        match response.get_ref().status() {
            ServingStatus::Serving => Ok(()),
            status => Err(format!(
                "grpc health check returned status {}",
                status.as_str_name()
            )),
        }
    }
}
