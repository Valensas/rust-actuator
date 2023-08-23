use std::collections::HashMap;

use rocket::{get, routes, serde::json::Json, Build, Rocket, State};

use crate::{
    actuator::RocketConfigurerer,
    health::{self, Check, IndicatorResult},
};

use super::Readiness;

#[get("/health/readiness")]
async fn readiness(
    readiness_check: &State<health::check::Readiness>,
) -> rocket::response::status::Custom<Json<HashMap<&str, IndicatorResult>>> {
    readiness_check.inner().check().await
}

impl health::check::Readiness {
    pub fn new(indicators: Vec<Box<dyn health::Indicator>>) -> Self {
        Self { indicators }
    }
}

impl crate::health::Check for Readiness {
    fn indicators(&self) -> &Vec<Box<dyn crate::health::Indicator>> {
        &self.indicators
    }
}

impl RocketConfigurerer for Readiness {
    fn configure(self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket.manage(self).mount("/", routes![readiness])
    }
}
