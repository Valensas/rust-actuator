use prometheus::{Registry, TextEncoder};
use rocket::{get, http::Status, routes, Build, Rocket, State};

use crate::actuator::RocketConfigurerer;

use super::Endpoint;

#[get("/metrics")]
fn metrics(registry: &State<Registry>) -> Result<String, rocket::response::status::Custom<String>> {
    let mut body = String::new();
    let encoder = TextEncoder::new();
    encoder
        .encode_utf8(&registry.gather(), &mut body)
        .map_err(|err| {
            rocket::response::status::Custom(
                Status::InternalServerError,
                format!("Could not collect metrics: {}", err),
            )
        })?;

    Ok(body)
}

impl Endpoint {
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }
}

impl RocketConfigurerer for Endpoint {
    fn configure(self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket.manage(self.registry).mount("/", routes![metrics])
    }
}
