use std::collections::HashMap;

use rocket::{get, routes, serde::json::Json, Build, Rocket, State};

use crate::{
    actuator::RocketConfigurerer,
    health,
    health::{Check, IndicatorResult},
};

#[get("/health/liveness")]
async fn liveness(
    liveness_check: &State<health::check::Liveness>,
) -> rocket::response::status::Custom<Json<HashMap<&str, IndicatorResult>>> {
    liveness_check.inner().check().await
}

impl health::check::Liveness {
    pub fn new(indicators: Vec<Box<dyn health::Indicator>>) -> Self {
        Self { indicators }
    }
}

impl health::Check for health::check::Liveness {
    fn indicators(&self) -> &Vec<Box<dyn health::Indicator>> {
        &self.indicators
    }
}

impl RocketConfigurerer for health::check::Liveness {
    fn configure(self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket.manage(self).mount("/", routes![liveness])
    }
}
